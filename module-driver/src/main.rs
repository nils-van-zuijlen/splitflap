#![no_std]
#![no_main]

use rp_pico::entry;

use rp_pico::hal::multicore::{Multicore, Stack};
use rp_pico::hal::pac;
use rp_pico::hal::uart::{self, UartConfig, UartPeripheral};
use rp_pico::hal::{self, Clock};

use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use embedded_hal::prelude::*;
use fugit::{ExtU32, RateExtU32};
use panic_halt as _;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

mod module_logic;
mod motor_driver;
mod parser;

static mut CORE1_STACK: Stack<4096> = Stack::new();

fn core1_thread() -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };

    let mut sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut count_down = timer.count_down();
    let mut delay = || {
        count_down.start(10.millis());
        let _ = nb::block!(count_down.wait());
    };

    // setup the rotor modules

    // module A
    let limit_sw = pins.gpio1.into_pull_up_input();
    let mut coil_a = pins.gpio28.into_push_pull_output();
    let mut coil_b = pins.gpio27.into_push_pull_output();
    let mut coil_c = pins.gpio26.into_push_pull_output();
    let mut coil_d = pins.gpio22.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_a = module_logic::Module::new(driver, &limit_sw, 635);

    // module B
    let limit_sw = pins.gpio2.into_pull_up_input();
    let mut coil_a = pins.gpio21.into_push_pull_output();
    let mut coil_b = pins.gpio20.into_push_pull_output();
    let mut coil_c = pins.gpio19.into_push_pull_output();
    let mut coil_d = pins.gpio18.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_b = module_logic::Module::new(driver, &limit_sw, 625);

    // module C
    let limit_sw = pins.gpio14.into_pull_up_input();
    let mut coil_a = pins.gpio6.into_push_pull_output();
    let mut coil_b = pins.gpio7.into_push_pull_output();
    let mut coil_c = pins.gpio8.into_push_pull_output();
    let mut coil_d = pins.gpio9.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_c = module_logic::Module::new(driver, &limit_sw, 625);

    // module D
    let limit_sw = pins.gpio15.into_pull_up_input();
    let mut coil_a = pins.gpio10.into_push_pull_output();
    let mut coil_b = pins.gpio11.into_push_pull_output();
    let mut coil_c = pins.gpio12.into_push_pull_output();
    let mut coil_d = pins.gpio13.into_push_pull_output();

    let driver = motor_driver::MotorDriver::new(&mut coil_a, &mut coil_b, &mut coil_c, &mut coil_d);
    let mut module_d = module_logic::Module::new(driver, &limit_sw, 625);

    // status LED
    let mut led = pins
        .gpio3
        .into_push_pull_output_in_state(hal::gpio::PinState::High);

    // reset barrels
    module_a.reset();
    module_b.reset();
    module_c.reset();
    module_d.reset();
    led.set_low().unwrap();

    loop {
        let modules = [&mut module_a, &mut module_b, &mut module_c, &mut module_d];

        // Each value in the FIFO contains the module index in the 16 MSB (with values 0b00, 0b01, 0b10, 0b11) and the letter index in the 8 LSB
        // Asking for flap 0xFF on the module will reset module immediately
        while let Some(command) = sio.fifo.read() {
            let module_index = (command >> 16) & 0b11;
            let letter_index: u8 = (command & 0xFF).try_into().unwrap();

            if letter_index == 0xFF {
                modules[module_index as usize].reset();
            } else {
                modules[module_index as usize].set_target(letter_index);
            }
        }

        led.toggle().unwrap();

        for module in modules {
            module.move_to_target().unwrap();
        }
        delay();
    }
}


#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut sio = hal::Sio::new(pac.SIO);

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

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // setup the HW UART buses
    let tx0 = pins.gpio16.into_mode();
    let rx0 = pins.gpio17.into_mode();

    let uart_lr = UartPeripheral::new(pac.UART0, (tx0, rx0), &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), uart::DataBits::Eight, None, uart::StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let tx1 = pins.gpio4.into_mode();
    let rx1 = pins.gpio5.into_mode();

    let uart_rl = UartPeripheral::new(pac.UART1, (tx1, rx1), &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), uart::DataBits::Eight, None, uart::StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let read_uart = uart_rl;
    let write_uart = uart_lr;

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
        .product("Split flap")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // Start core1_thread
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _task = core1.spawn(unsafe { &mut CORE1_STACK.mem }, core1_thread);

    sio.fifo.write((0 << 16) + 0); // blank
    sio.fifo.write((1 << 16) + 0); // blank
    sio.fifo.write((2 << 16) + 0); // blank
    sio.fifo.write((3 << 16) + 0); // blank

    let mut parser = parser::Parser::new(sio.fifo, write_uart);

    loop {
        // Check for new data on the USB bus
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
                    buf.iter().take(count)
                        .for_each(|b| parser.parse_char(*b));
                }
            }
        }

        // Check for new data on the receiving UART
        if read_uart.uart_is_readable() {
            let mut buf = [0u8; 64];
            match read_uart.read_raw(&mut buf) {
                Err(_) => {}
                Ok(0) => {}
                Ok(count) => {
                    buf.iter().take(count)
                        .for_each(|b| parser.parse_char(*b));
                }
            }
        }
    }
}
