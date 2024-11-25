//! High level hidapi abstraction for interacting with zoom65v3 screen modules

use std::sync::{LazyLock, RwLock};

use checksum::checksum;
use chrono::{DateTime, Datelike, TimeZone, Timelike};
use float::DumbFloat16;
use hidapi::{HidApi, HidDevice};
use types::{ScreenPosition, ScreenTheme, UploadChannel, Zoom65Result};

use crate::types::{Icon, Zoom65Error};

pub mod abi;
pub mod checksum;
pub mod consts;
pub mod float;
pub mod types;

/// Lazy handle to hidapi
static API: LazyLock<RwLock<HidApi>> =
    LazyLock::new(|| RwLock::new(HidApi::new().expect("failed to init hidapi")));

/// High level abstraction for managing a zoom65 v3 keyboard
pub struct Zoom65v3 {
    pub device: HidDevice,
    buf: [u8; 64],
}

impl Zoom65v3 {
    /// Find and open the device for modifications
    pub fn open() -> Result<Self, Zoom65Error> {
        API.write().unwrap().refresh_devices()?;
        let api = API.read().unwrap();
        let mut this = Self {
            device: api
                .device_list()
                .find(|d| {
                    d.vendor_id() == consts::ZOOM65_VENDOR_ID
                        && d.product_id() == consts::ZOOM65_PRODUCT_ID
                        && d.usage_page() == consts::ZOOM65_USAGE_PAGE
                        && d.usage() == consts::ZOOM65_USAGE
                })
                .ok_or(Zoom65Error::DeviceNotFound)?
                .open_device(&api)?,
            buf: [0u8; 64],
        };

        if !consts::APPROVED_VERSIONS.contains(&this.get_version()?) {
            return Err(Zoom65Error::UnknownFirmwareVersion);
        }
        Ok(this)
    }

    /// Internal method to execute a payload and read the response
    fn execute(&mut self, payload: [u8; 33]) -> Zoom65Result<Vec<u8>> {
        self.device.write(&payload)?;
        let len = self.device.read(&mut self.buf)?;
        let slice = &self.buf[..len];
        assert!(slice[0] == payload[1]);
        Ok(slice.to_vec())
    }

    /// Get the version id tracked by the web driver
    #[inline(always)]
    pub fn get_version(&mut self) -> Zoom65Result<u8> {
        let res = self.execute(abi::get_version())?;
        // Return the version byte (at least, the one that the web driver tracks)
        Ok(res[2])
    }

    /// Set the screen theme. Will reset the screen back to the meletrix logo
    #[inline(always)]
    pub fn screen_theme(&mut self, theme: ScreenTheme) -> Zoom65Result<()> {
        let res = self.execute(abi::screen_theme(theme))?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Increment the screen position
    #[inline(always)]
    pub fn screen_up(&mut self) -> Zoom65Result<()> {
        let res = self.execute(abi::screen_up())?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Decrement the screen position
    #[inline(always)]
    pub fn screen_down(&mut self) -> Zoom65Result<()> {
        let res = self.execute(abi::screen_down())?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Switch the active screen
    #[inline(always)]
    pub fn screen_switch(&mut self) -> Zoom65Result<()> {
        let res = self.execute(abi::screen_switch())?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Reset the screen back to the meletrix logo
    #[inline(always)]
    pub fn reset_screen(&mut self) -> Zoom65Result<()> {
        let res = self.execute(abi::reset_screen())?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Set the screen to a specific position and offset
    pub fn set_screen(&mut self, position: ScreenPosition) -> Zoom65Result<()> {
        let (y, x) = position.to_directions();

        // Back to default
        self.reset_screen()?;

        // Move screen up or down
        match y {
            y if y < 0 => {
                for _ in 0..y.abs() {
                    self.screen_up()?;
                }
            },
            y if y > 0 => {
                for _ in 0..y.abs() {
                    self.screen_down()?;
                }
            },
            _ => {},
        }

        // Switch screen to offset
        for _ in 0..x {
            self.screen_switch()?;
        }

        Ok(())
    }

    /// Update the keyboards current time
    #[inline(always)]
    pub fn set_time<Tz: TimeZone>(&mut self, time: DateTime<Tz>) -> Zoom65Result<()> {
        let res = self.execute(abi::set_time(
            // Provide the current year without the century.
            // This prevents overflows on the year 2256 (meletrix web ui just subtracts 2000)
            (time.year() % 100) as u8,
            time.month() as u8,
            time.day() as u8,
            time.hour() as u8,
            time.minute() as u8,
            time.second() as u8,
        ))?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Update the keyboards current weather report
    #[inline(always)]
    pub fn set_weather(&mut self, icon: Icon, current: u8, low: u8, high: u8) -> Zoom65Result<()> {
        let res = self.execute(abi::set_weather(icon, current, low, high))?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Update the keyboards current system info
    #[inline(always)]
    pub fn set_system_info(
        &mut self,
        cpu_temp: u8,
        gpu_temp: u8,
        download_rate: f32,
    ) -> Zoom65Result<()> {
        let download = DumbFloat16::new(download_rate);
        let res = self.execute(abi::set_system_info(cpu_temp, gpu_temp, download))?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    fn upload_media(
        &mut self,
        buf: impl AsRef<[u8]>,
        channel: UploadChannel,
        cb: impl Fn(usize),
    ) -> Zoom65Result<()> {
        let image = buf.as_ref();

        // start upload
        let res = self.execute(abi::upload_start(channel))?;
        if res[1] != 1 || res[2] != 1 {
            return Err(Zoom65Error::UpdateCommandFailed);
        }
        let res = self.execute(abi::upload_length(image.len() as u32))?;
        if res[1] != 1 || res[2] != 1 {
            return Err(Zoom65Error::UpdateCommandFailed);
        }

        for (i, chunk) in image.chunks(24).enumerate() {
            cb(i);

            let chunk_len = chunk.len();
            let mut buf = [0u8; 33];

            // command prefix
            buf[0] = 0x0;
            buf[1] = 88;
            buf[2] = 2 + chunk_len as u8 + 4;

            // chunk index and data
            buf[3] = (i >> 8) as u8;
            buf[4] = (i & 255) as u8;
            buf[5..5 + chunk.len()].copy_from_slice(chunk);

            let mut offset = 3 + 2 + chunk_len;

            // Images are always aligned, but we need to manually align the last chunk of gifs
            if channel == UploadChannel::Gif && i == image.len() / 24 {
                // compute padding for final payload, the checksum needs 32-bit alignment
                let padding = (4 - (image.len() % 24) % 4) % 4;
                buf[2] += padding as u8;
                offset += padding;
            }

            // compute checksum
            let data = &buf[3..offset + 2];
            let crc = checksum(data);
            buf[offset..offset + 4].copy_from_slice(&crc);

            // send payload and read response
            let res = self.execute(buf)?;
            if res[1] != 1 || res[2] != 1 {
                return Err(Zoom65Error::UpdateCommandFailed);
            }
        }

        let res = self.execute(abi::upload_end())?;
        if res[1] != 1 || res[2] != 1 {
            return Err(Zoom65Error::UpdateCommandFailed);
        }

        // TODO: is this required?
        self.reset_screen()?;

        println!("done");

        Ok(())
    }

    /// Upload an image to the keyboard. Must be encoded as 110x110 RGBA-3328 raw buffer
    #[inline(always)]
    pub fn upload_image(&mut self, buf: impl AsRef<[u8]>, cb: impl Fn(usize)) -> Zoom65Result<()> {
        let buf = buf.as_ref();
        if buf.len() != 36300 {
            return Err(Zoom65Error::GifTooLarge);
        }
        self.upload_media(buf, UploadChannel::Image, cb)
    }

    /// Upload a gif to the keyboard. Must be 111x111.
    #[inline(always)]
    pub fn upload_gif(&mut self, buf: impl AsRef<[u8]>, cb: impl Fn(usize)) -> Zoom65Result<()> {
        if buf.as_ref().len() >= 1013808 {
            return Err(Zoom65Error::GifTooLarge);
        }
        self.upload_media(buf, UploadChannel::Gif, cb)
    }

    /// Clear the image slot
    #[inline(always)]
    pub fn clear_image(&mut self) -> Zoom65Result<()> {
        let res = self.execute(abi::delete_image())?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }

    /// Clear the gif slot
    #[inline(always)]
    pub fn clear_gif(&mut self) -> Zoom65Result<()> {
        let res = self.execute(abi::delete_gif())?;
        (res[1] == 1 && res[2] == 1)
            .then_some(())
            .ok_or(Zoom65Error::UpdateCommandFailed)
    }
}
