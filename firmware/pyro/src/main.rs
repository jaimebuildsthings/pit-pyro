//! Firmware for the pit-pyro handheld tire pyrometer, on a FeatherS3[D].
//!
//! Desk-test stage: reads the MLX90640 thermal camera on STEMMA QT port 2 and
//! prints its images at the serial console, over the board's USB-C port.
//! Type `help`.

#![no_std]
#![no_main]

mod app;
mod console;
mod out;

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::usb::usb_serial_jtag::UsbSerialJtag;
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("\r\npanic: {info}\r\nrestarting");
    // Let the USB port drain before the reset cuts the message off.
    esp_hal::delay::Delay::new().delay_millis(1000);
    esp_hal::system::software_reset()
}

/// I2C clock. The MLX90640 allows 1 MHz; 400 kHz leaves margin for the
/// 150 mm cable and the sensor's 4.7 kΩ pull-ups.
const I2C_KHZ: u32 = 400;

#[esp_hal::main]
async fn main(spawner: Spawner) {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let timg0 = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timg0.timer0, p.FROM_CPU_INTR0);

    // The console: the Feather's USB-C port (the ESP32-S3's USB Serial/JTAG).
    let (rx, tx) = UsbSerialJtag::new(p.USB_DEVICE).into_async().split();
    spawner.spawn(out::task(tx).expect("output task"));
    spawner.spawn(console::task(rx).expect("console task"));

    // LDO2 powers STEMMA QT port 2 and turns off in deep sleep; IO39 switches it on.
    let ldo2 = Output::new(p.GPIO39, Level::High, OutputConfig::default());

    // Pin map; see firmware/README.md. STEMMA QT port 1 also carries the
    // Feather's battery gauge and has 10 kΩ pull-ups on the board; port 2 has
    // none, so the sensor carrier has its own.
    let config = Config::default().with_frequency(Rate::from_khz(I2C_KHZ));
    let sensor_bus = I2c::new(p.I2C0, config).expect("I2C0 config").with_sda(p.GPIO16).with_scl(p.GPIO15);
    let stemma1 = I2c::new(p.I2C1, config).expect("I2C1 config").with_sda(p.GPIO8).with_scl(p.GPIO9);

    // About 9 KB of images and subpages: kept out of the task's stack.
    static APP: StaticCell<app::App> = StaticCell::new();
    APP.init(app::App::new(sensor_bus, stemma1, ldo2)).run().await
}
