// lib.rs
#![warn(clippy::large_futures)]

#[cfg(all(feature = "display-max7219", feature = "display-ws2812"))]
compile_error!(
    "Enable at most one display backend feature: 'display-max7219' or 'display-ws2812'."
);

pub use std::{
    any::Any,
    fmt, net,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

pub use anyhow::bail;
pub use askama::Template;
pub use chrono::*;
#[allow(ambiguous_glob_reexports)]
pub use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    gpio::{self, *},
    peripherals::Peripherals,
    spi,
    units::FromValueType,
};
pub use esp_idf_svc::{hal::spi::SpiDeviceDriver, nvs, sntp, wifi::WifiDriver};
#[cfg(feature = "display-max7219")]
use max7219::{MAX7219, connectors::SpiConnector};
pub use serde::{Deserialize, Serialize};
pub use tokio::{
    sync::{Mutex, RwLock},
    time::{Duration, sleep, timeout},
};
pub use tracing::*;

mod config;
pub use config::*;

mod state;
pub use state::*;

mod measure;
pub use measure::*;

mod rmt_ow;
pub use rmt_ow::*;

mod mqtt;
pub use mqtt::*;

mod apiserver;
pub use apiserver::*;

mod wifi;
pub use wifi::*;

#[cfg(any(feature = "display-max7219", feature = "display-ws2812"))]
mod display;
#[cfg(any(feature = "display-max7219", feature = "display-ws2812"))]
pub use display::*;

#[cfg(any(feature = "display-max7219", feature = "display-ws2812"))]
mod font;

#[cfg(feature = "display-ws2812")]
mod ws2812;
#[cfg(feature = "display-ws2812")]
pub use ws2812::LedMatrix;

#[cfg(feature = "display-max7219")]
pub type LedMatrix<'a> = MAX7219<SpiConnector<SpiDeviceDriver<'a, spi::SpiDriver<'a>>>>;

pub const FW_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AP_MODE_SSID: &str = "esp32example";
pub const AP_MODE_IP_ADDR: net::Ipv4Addr = net::Ipv4Addr::new(10, 42, 42, 1);
pub const AP_MODE_IP_MASK: u8 = 24;

#[cfg(feature = "esp32-c3")]
pub const LED_ACTIVE_LOW: bool = true;
#[cfg(feature = "esp-wroom-32")]
pub const LED_ACTIVE_LOW: bool = false;

pub const NO_TEMP: f32 = -1000.0;

#[derive(Clone, Debug, Serialize)]
pub struct TempData {
    pub iopin: String,
    pub sensor: String,
    pub value: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Sensor {
    pub iopin: String,
    pub sensor: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TempValues {
    pub timestamp: i64,
    pub last_update: String,
    pub uptime: u32,
    pub uptime_s: String,
    pub temperatures: Vec<TempData>,
}

impl TempValues {
    pub fn new() -> Self {
        TempValues {
            timestamp: 0,
            last_update: "-".to_string(),
            uptime: 0,
            uptime_s: "-".to_string(),
            temperatures: Vec::new(),
        }
    }

    pub fn with_capacity(c: usize) -> Self {
        TempValues {
            timestamp: 0,
            last_update: "-".to_string(),
            uptime: 0,
            uptime_s: "-".to_string(),
            temperatures: Vec::with_capacity(c),
        }
    }
}

impl Default for TempValues {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SensorValues {
    pub sensors: Vec<Sensor>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Uptime {
    pub uptime: u32,
    pub uptime_s: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SampleState {
    pub counter: u32,
    pub source: String,
    pub message: String,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct SampleMessage {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFirmware {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct LedCommand {
    pub on: bool,
    pub duration_ms: Option<u64>,
}

// EOF
