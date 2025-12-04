#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::timer::timg::TimerGroup;

// Logger.
use esp_println::logger;
use log::info;

// For LED.
// use esp_hal::gpio::{Level, Output, OutputConfig};

// For BME280.
use bme280::i2c::BME280;
use embedded_hal::delay::DelayNs;
use esp_hal::i2c::master::{Config, I2c};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.1

    logger::init_logger(log::LevelFilter::Info);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    /*
    // High is when LED is off.
    let mut led = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());
    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
        led.toggle();
        info!("pin level is {:?}", led.output_level());
    }
    */

    // This code is for ESP32C3-Supermini.
    let i2c_bus = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);
    let mut bme280 = BME280::new_primary(i2c_bus);
    let mut delay = Delay::new();
    bme280.init(&mut delay).unwrap();

    loop {
        let result = bme280.measure(&mut delay).unwrap();

        info!(
            "Temperature: {} degC;  Pressure: {} Pa;  Humidity: {}%",
            result.temperature, result.pressure, result.humidity
        );

        delay.delay_ms(1000);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
