use std::sync::LazyLock;

use chrono::{DateTime, Datelike, TimeZone, Timelike};
use hidapi::{HidApi, HidDevice, HidError};

use crate::weather::Icon;

mod consts;

/// Lazy handle to hidapi
static API: LazyLock<HidApi> = LazyLock::new(|| HidApi::new().expect("failed to init hidapi"));

#[derive(thiserror::Error)]
pub enum Zoom65Error {
    #[error("failed to find device")]
    DeviceNotFound,
    #[error("firmware version is unknown. open an issue for support")]
    UnknownFirmwareVersion,
    #[error("keyboard responded with error while updating, byte 1 == 88 && byte 2 == 0")]
    UpdateCommandFailed,
    #[error("hid device error: {_0}")]
    Hid(#[from] HidError),
}

impl std::fmt::Debug for Zoom65Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

/// High level handle for managing a zoom65 v3 keyboard
pub(crate) struct Zoom65v3 {
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
    fn update(&mut self, slot: u8, slice: &[u8]) -> Result<(), Zoom65Error> {
        // Construct command sequence
        let mut buf = [0u8; 33];
        buf[0] = 0x0;
        buf[1] = 88;
        buf[2] = slice.len() as u8 + 3;
        buf[3] = 165;
        buf[4] = 1;
        buf[5] = slot;
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

    /// Update the keyboards current time
    pub fn set_time<Tz: TimeZone>(&mut self, time: DateTime<Tz>) -> Result<(), Zoom65Error> {
        self.update(
            16,
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
        self.update(32, &[icon as u8, current, low, high])
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
            64,
            &[cpu_temp, gpu_temp, download_m as u8, download_r as u8],
        )
    }
}
