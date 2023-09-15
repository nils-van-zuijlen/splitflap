#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::{gpio, rtc, i2c, bind_interrupts};
use embassy_time::{Duration, Ticker, Delay, Instant};
use gpio::{Level, Output, Input, Pull};
use panic_halt as _;
use embassy_sync::mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_rp::peripherals::{UART0, RTC, I2C1};
use embassy_rp::uart::{self, Async, UartTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use hd44780_driver::non_blocking::HD44780;
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode};

mod menu;

const MESSAGE_LENGTH: usize = 5;


static SETTINGS: Mutex<CriticalSectionRawMutex, menu::Settings> = Mutex::new(menu::Settings::new());
static DISPLAY: Mutex<CriticalSectionRawMutex, menu::Disp> = Mutex::new(menu::Disp::new());
static HOUR_SETTING: Channel<CriticalSectionRawMutex, menu::TimeSetting, 1> = Channel::new();
static DISPLAY_CHANGE: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static DISPLAY_RESET: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

#[embassy_executor::task]
async fn uart_output_task(mut tx: UartTx<'static, UART0, Async>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));

    let reset_msg = [0xFF];
    let mut buf = [0; MESSAGE_LENGTH + 1];
    buf[0] = 0xF0; // word start

    let mut last_switch = Instant::now();
    let mut last_reset = Instant::now();
    tx.write(&reset_msg).await.unwrap();

    loop {
        ticker.next().await;

        let (message, should_reset, switched) = {
            let mut settings = SETTINGS.lock().await;

            (settings.get_message(), settings.should_reset(last_reset), settings.switch_message(last_switch))
        };

        if should_reset {
            last_reset = Instant::now();
            tx.write(&reset_msg).await.unwrap();
        }
        match switched {
            Some(true) | None => last_switch = Instant::now(),
            Some(false) => {}
        };

        buf[1..(MESSAGE_LENGTH + 1)].copy_from_slice(&message[..MESSAGE_LENGTH]);

        tx.write(&buf).await.unwrap();
    }
}

#[embassy_executor::task]
async fn rtc_updater(mut rtc: rtc::Rtc<'static, RTC>) {
    let mut ticker = Ticker::every(Duration::from_hz(10));

    if !rtc.is_running() {
        let settings = SETTINGS.lock().await;
        rtc.set_datetime(settings.current_time().clone()).unwrap();
    }

    loop {
        ticker.next().await;

        if let Ok(change) = HOUR_SETTING.try_recv() {
            let mut now = rtc.now().unwrap();
            match change {
                menu::TimeSetting::Hour(new_hour) => {now.hour = new_hour},
                menu::TimeSetting::Minute(new_minute) => {now.minute = new_minute},
                menu::TimeSetting::Second(new_second) => {now.second = new_second}
            };
            rtc.set_datetime(now).unwrap();
        };

        if let Ok(now) = rtc.now() {
            let mut settings = SETTINGS.lock().await;
            if settings.set_current_time(now) {
                DISPLAY_CHANGE.signal(());
            }
        }
    }
}

#[embassy_executor::task]
async fn inputs_task(enc_clk: Input<'static, gpio::AnyPin>, enc_dir: Input<'static, gpio::AnyPin>, enc_btn: Input<'static, gpio::AnyPin>, back_btn: Input<'static, gpio::AnyPin>) {
    let mut ticker = Ticker::every(Duration::from_hz(800));

    let mut last_clk = enc_clk.get_level();
    let mut last_btn = enc_btn.get_level();
    let mut last_back = back_btn.get_level();

    loop {

        ticker.next().await;
        let cur_clk = enc_clk.get_level();
        if last_clk == Level::High && cur_clk == Level::Low { // falling edge
            let mut display = DISPLAY.lock().await;
            let mut settings = SETTINGS.lock().await;
            if enc_dir.is_high() {
                display.up(&mut settings)
            } else {
                display.down(&mut settings)
            }
            DISPLAY_CHANGE.signal(());
        }
        last_clk = cur_clk;

        let cur_btn = enc_btn.get_level();
        if last_btn == Level::High && cur_btn == Level::Low { // falling edge
            let mut display = DISPLAY.lock().await;
            let mut settings = SETTINGS.lock().await;
            display.enter(&mut settings).await;
        }
        last_btn = cur_btn;

        let cur_back = back_btn.get_level();
        if last_back == Level::High && cur_back == Level::Low { // falling edge
            DISPLAY_RESET.signal(());
            let mut display = DISPLAY.lock().await;
            display.back();
        }
        last_back = cur_back;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // uart sender
    let mut uart_config = uart::Config::default();
    uart_config.baudrate = 9600;
    let uart = UartTx::new(p.UART0, p.PIN_12, p.DMA_CH0, uart_config);
    spawner.spawn(uart_output_task(uart)).unwrap();

    // rtc
    let rtc = rtc::Rtc::new(p.RTC);
    spawner.spawn(rtc_updater(rtc)).unwrap();

    // encoder
    let enc_clk = Input::new(gpio::AnyPin::from(p.PIN_20), Pull::Up);
    let enc_dir = Input::new(gpio::AnyPin::from(p.PIN_21), Pull::Up);
    let enc_btn = Input::new(gpio::AnyPin::from(p.PIN_19), Pull::Up);
    let back_btn = Input::new(gpio::AnyPin::from(p.PIN_18), Pull::Up);
    spawner.spawn(inputs_task(enc_clk, enc_dir, enc_btn, back_btn)).unwrap();

    let _led = Output::new(p.PIN_25, Level::High);

    // display
    let scl = p.PIN_3;
    let sda = p.PIN_2;
    let i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, i2c::Config::default());
    let mut delay = Delay;
    let mut lcd = HD44780::new_i2c(i2c, 0x3F, &mut delay).await.unwrap();
    DISPLAY_RESET.signal(());

    loop {
        DISPLAY_CHANGE.wait().await;
        DISPLAY_CHANGE.reset();
        let (lines, display_off) = {
            let display = DISPLAY.lock().await;
            let settings = SETTINGS.lock().await;
            (display.output(&settings), match display.screen() {
                menu::Screens::Home(menu::HomePosition::DisplayCurrentOff) => Display::Off,
                _ => Display::On
            })
        };

        if DISPLAY_RESET.signaled() {
            DISPLAY_RESET.reset();
            lcd.reset(&mut delay).await.unwrap();
            lcd.clear(&mut delay).await.unwrap();
            lcd.set_display_mode(
                DisplayMode {
                    display: Display::On,
                    cursor_visibility: Cursor::Invisible,
                    cursor_blink: CursorBlink::Off,
                }, &mut delay).await.unwrap(
            );
        }


        lcd.set_cursor_pos(0, &mut delay).await.unwrap();
        lcd.write_str(&lines[0], &mut delay).await.unwrap();
        let remaining = 20 - lines[0].len();
        for _ in 0..remaining {
            lcd.write_char(' ', &mut delay).await.unwrap();
        }
        lcd.write_str(&lines[2], &mut delay).await.unwrap();
        let remaining = 20 - lines[2].len();
        for _ in 0..remaining {
            lcd.write_char(' ', &mut delay).await.unwrap();
        }
        lcd.write_str(&lines[1], &mut delay).await.unwrap();
        let remaining = 20 - lines[1].len();
        for _ in 0..remaining {
            lcd.write_char(' ', &mut delay).await.unwrap();
        }
        lcd.write_str(&lines[3], &mut delay).await.unwrap();
        let remaining = 20 - lines[3].len();
        for _ in 0..remaining {
            lcd.write_char(' ', &mut delay).await.unwrap();
        }

        lcd.set_display(display_off, &mut delay).await.unwrap();
    }
}
