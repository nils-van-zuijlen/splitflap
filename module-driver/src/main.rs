#![no_std]
#![no_main]

use rp_pico::entry;

use rp_pico::hal::{self, Clock};
use rp_pico::hal::uart::{self, UartPeripheral, UartConfig};
use rp_pico::hal::pac;

use embedded_hal::prelude::*;
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
use fugit::{RateExtU32, ExtU32};

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;
use core::fmt::Write;
use heapless::String;

mod motor_driver;
mod module_logic;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let sio = hal::Sio::new(pac.SIO);

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut count_down = timer.count_down();
    let mut delay = || {
        count_down.start(10.millis());
        let _ = nb::block!(count_down.wait());
    };

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    // setup the HW UART buses
    let tx0 = pins.gpio16.into_mode();
    let rx0 = pins.gpio17.into_mode();

    let uart_lr = UartPeripheral::new(pac.UART0, (tx0, rx0), &mut pac.RESETS).enable(UartConfig::new(9600.Hz(), uart::DataBits::Eight, None, uart::StopBits::One), clocks.peripheral_clock.freq()).unwrap();

    let tx1 = pins.gpio4.into_mode();
    let rx1 = pins.gpio5.into_mode();

    let uart_rl = UartPeripheral::new(pac.UART1, (tx1, rx1), &mut pac.RESETS).enable(UartConfig::new(9600.Hz(), uart::DataBits::Eight, None, uart::StopBits::One), clocks.peripheral_clock.freq()).unwrap();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut usb_serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // setup the rotor modules

    // module A
    let limit_sw = pins.gpio1.into_pull_up_input();
    let mut coil_a = pins.gpio28.into_push_pull_output();
    let mut coil_b = pins.gpio27.into_push_pull_output();
    let mut coil_c = pins.gpio26.into_push_pull_output();
    let mut coil_d = pins.gpio22.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_a = module_logic::Module::new(driver, &limit_sw, 20);

    // module B
    let limit_sw = pins.gpio2.into_pull_up_input();
    let mut coil_a = pins.gpio21.into_push_pull_output();
    let mut coil_b = pins.gpio20.into_push_pull_output();
    let mut coil_c = pins.gpio19.into_push_pull_output();
    let mut coil_d = pins.gpio18.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_b = module_logic::Module::new(driver, &limit_sw, 20);

    // module C
    let limit_sw = pins.gpio14.into_pull_up_input();
    let mut coil_a = pins.gpio6.into_push_pull_output();
    let mut coil_b = pins.gpio7.into_push_pull_output();
    let mut coil_c = pins.gpio8.into_push_pull_output();
    let mut coil_d = pins.gpio9.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_c = module_logic::Module::new(driver, &limit_sw, 20);

    // module D
    let limit_sw = pins.gpio15.into_pull_up_input();
    let mut coil_a = pins.gpio10.into_push_pull_output();
    let mut coil_b = pins.gpio11.into_push_pull_output();
    let mut coil_c = pins.gpio12.into_push_pull_output();
    let mut coil_d = pins.gpio13.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_d = module_logic::Module::new(driver, &limit_sw, 20);


    // status LED
    let mut led = pins.gpio3.into_push_pull_output();

    // reset barrels
    led.set_high().unwrap();
    module_a.reset(&mut delay).unwrap();
    module_b.reset(&mut delay).unwrap();
    module_c.reset(&mut delay).unwrap();
    module_d.reset(&mut delay).unwrap();
    led.set_low().unwrap();

    while true {

        // Check for new data
        if usb_dev.poll(&mut [&mut usb_serial]) {
            let mut buf = [0u8; 64];
            match usb_serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    // Convert to upper case
                    buf.iter_mut().take(count).for_each(|b| {
                        b.make_ascii_uppercase();
                    });
                    // Send back to the host
                    let mut wr_ptr = &buf[..count];
                    while !wr_ptr.is_empty() {
                        match usb_serial.write(wr_ptr) {
                            Ok(len) => wr_ptr = &wr_ptr[len..],
                            // On error, just drop unwritten data.
                            // One possible error is Err(WouldBlock), meaning the USB
                            // write buffer is full.
                            Err(_) => break,
                        };
                    }
                }
            }
        }
    }
//*/


    loop {
        for asked in 0..module_logic::LETTER_COUNT {
            for _ in 0..100 {
            delay();
            }
            module_a.set_target(asked);
            while module_a.moving() {
                module_a.move_to_target().unwrap();
                delay();
            }
        }
    }
}
