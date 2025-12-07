#![no_std]

use bme280::i2c::BME280;
use embedded_hal::delay::DelayNs;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::Blocking;
use esp_radio::wifi::{ClientConfig, ModeConfig, WifiController, WifiError};
use log::info;

fn wifi_wait_for<F>(mut f: F) -> Result<(), WifiError>
where
    F: FnMut() -> Result<bool, WifiError>,
{
    loop {
        let res = f();
        match res {
            Ok(ok) => {
                if ok {
                    return Ok(());
                }
            }
            Err(err) => return Err(err),
        }
        let mut delay = Delay::new();
        delay.delay_ms(500);
    }
}

pub fn start_wifi_client(wifi: &mut WifiController) -> Result<(), WifiError> {
    const WIFI_ID: &str = env!("WIFI_ID");
    const WIFI_PASS: &str = env!("WIFI_PASS");
    let wifi_conf = ModeConfig::Client(
        ClientConfig::default()
            .with_ssid(WIFI_ID.into())
            .with_password(WIFI_PASS.into()),
    );
    wifi.set_config(&wifi_conf)?;
    info!("wifi config is set");
    wifi.start()?;
    wifi_wait_for(|| wifi.is_started())?;
    info!("WiFi is started");
    wifi.connect()?;
    wifi_wait_for(|| wifi.is_connected())?;
    info!("WiFi is connected to {WIFI_ID}");
    Ok(())
}

pub fn run_led(pin: impl esp_hal::gpio::OutputPin) -> ! {
    // High is when LED is off.
    let mut led = Output::new(pin, Level::High, OutputConfig::default());
    let mut delay = Delay::new();
    loop {
        delay.delay_ms(1000);
        led.toggle();
        info!("pin level is {:?}", led.output_level());
    }
}

pub fn run_bme280_reader(i2c_bus: esp_hal::i2c::master::I2c<Blocking>) -> ! {
    // This code is for ESP32C3-Supermini.
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
}
