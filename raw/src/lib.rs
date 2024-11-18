//! High level abstraction for interacting with zoom65v3 screen modules

use std::sync::LazyLock;

use chrono::{DateTime, Datelike, TimeZone, Timelike};
use consts::commands;
use hidapi::{HidApi, HidDevice};

use crate::types::Icon;
use crate::types::Zoom65Error;

pub mod consts;
pub mod float;
pub mod types;

/// Lazy handle to hidapi
static API: LazyLock<HidApi> = LazyLock::new(|| HidApi::new().expect("failed to init hidapi"));

/// High level abstraction for managing a zoom65 v3 keyboard
pub struct Zoom65v3 {
    device: HidDevice,
    buf: [u8; 64],
}

impl Zoom65v3 {
    /// Find and open the device for modifications
    pub fn open() -> Result<Self, Zoom65Error> {
        let mut this = Self {
            device: API
                .device_list()
                .find(|d| {
                    d.vendor_id() == consts::ZOOM65_VENDOR_ID
                        && d.product_id() == consts::ZOOM65_PRODUCT_ID
                        && d.usage_page() == consts::ZOOM65_USAGE_PAGE
                        && d.usage() == consts::ZOOM65_USAGE
                })
                .ok_or(Zoom65Error::DeviceNotFound)?
                .open_device(&API)?,
            buf: [0u8; 64],
        };

        if !consts::APPROVED_VERSIONS.contains(&this.get_version()?) {
            return Err(Zoom65Error::UnknownFirmwareVersion);
        }
        Ok(this)
    }

    /// Get the version id tracked by the web driver
    pub fn get_version(&mut self) -> Result<u8, Zoom65Error> {
        // Write to device and read response
        self.device.write(&consts::commands::ZOOM65_VERSION_CMD)?;
        let len = self.device.read(&mut self.buf)?;
        let slice = &self.buf[..len];
        assert!(slice[0] == 1);

        // Return the version byte (at least, the one that the web driver tracks)
        Ok(slice[2])
    }

    /// Internal method to send and parse an update command
    fn update(&mut self, method_id: [u8; 2], slice: &[u8]) -> Result<(), Zoom65Error> {
        // Construct command sequence
        let mut buf = [0u8; 33];
        buf[0] = 0x0;
        buf[1] = 88;
        buf[2] = slice.len() as u8 + 3;
        buf[3] = 165;
        buf[4] = method_id[0];
        buf[5] = method_id[1];
        buf[6..6 + slice.len()].copy_from_slice(slice);

        // Write to device and read response
        self.device.write(&buf)?;
        let len = self.device.read(&mut self.buf)?;
        let slice = &self.buf[..len];
        assert!(slice[0] == 88);

        // Return result based on output code
        (slice[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Increment the screen position
    #[inline(always)]
    pub fn screen_up(&mut self) -> Result<(), Zoom65Error> {
        self.update(commands::ZOOM65_SCREEN_UP, &[])
    }

    /// Decrement the screen position
    #[inline(always)]
    pub fn screen_down(&mut self) -> Result<(), Zoom65Error> {
        self.update(commands::ZOOM65_SCREEN_DOWN, &[])
    }

    /// Switch the active screen
    #[inline(always)]
    pub fn screen_switch(&mut self) -> Result<(), Zoom65Error> {
        self.update(commands::ZOOM65_SCREEN_SWITCH, &[])
    }

    /// Update the keyboards current time
    pub fn set_time<Tz: TimeZone>(&mut self, time: DateTime<Tz>) -> Result<(), Zoom65Error> {
        self.update(
            commands::ZOOM65_SET_TIME_ID,
            &[
                // Provide the current year without the century.
                // This prevents overflows on the year 2256 (meletrix web ui just subtracts 2000)
                (time.year() % 100) as u8,
                time.month() as u8,
                time.day() as u8,
                time.hour() as u8,
                time.minute() as u8,
                time.second() as u8,
            ],
        )
    }

    /// Update the keyboards current weather report
    pub fn set_weather(
        &mut self,
        icon: Icon,
        current: u8,
        low: u8,
        high: u8,
    ) -> Result<(), Zoom65Error> {
        self.update(
            commands::ZOOM65_SET_WEATHER_ID,
            &[icon as u8, current, low, high],
        )
    }

    /// Update the keyboards current system info
    pub fn set_system_info(
        &mut self,
        cpu_temp: u8,
        gpu_temp: u8,
        download_rate: f64,
    ) -> Result<(), Zoom65Error> {
        // This is my best guess at how they are encoding the download float into 2 bytes
        //
        // Sending the following sets the display to:
        //   [0, 0] => 0.00
        //   [0, 1] => 0.01
        //   [1, 0] => 2.56
        //   [1, 1] => 2.57
        //
        // With the below solution:
        //   f(100) =>  99.84
        //   f(200) => 199.68
        let download_m = download_rate / 2.56;
        let download_r = download_rate % 2.56;

        self.update(
            commands::ZOOM65_SET_SYSINFO_ID,
            &[cpu_temp, gpu_temp, download_m as u8, download_r as u8],
        )
    }

    /// Reset the screen back to the meletrix logo
    pub fn reset_screen(&mut self) -> Result<(), Zoom65Error> {
        self.update(commands::ZOOM65_RESET_SCREEN_ID, &[])
    }
}
