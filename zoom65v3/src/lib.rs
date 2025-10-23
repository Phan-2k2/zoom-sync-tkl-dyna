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
pub mod float;
pub mod types;

pub mod consts {
    pub const ZOOM65_VENDOR_ID: u16 = 0x5542;
    pub const ZOOM65_PRODUCT_ID: u16 = 0xC987;
    pub const ZOOM65_USAGE_PAGE: u16 = 65376;
    pub const ZOOM65_USAGE: u16 = 97;
}

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
        let this = Self {
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
        println!("Device Found: {:?}", this.device.get_product_string().unwrap().unwrap());

        Ok(this)
    }

    /// Internal method to execute a payload and read the response
    fn execute(&mut self, payload: [u8; 33]) -> Zoom65Result<Vec<u8>> {
        self.device.write(&payload)?;
        println!("{:x?}", payload);
        let len = self.device.read(&mut self.buf)?;
        let slice = &self.buf[0..len];
        println!("{:x?}", slice);
        assert!(slice[0] == payload[1]);
        Ok(slice.to_vec())
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

    /// Update the keyboards current time.
    /// If 12hr is true, hardcodes the time to 01:00-12:00 for the current day.
    #[inline(always)]
    pub fn set_time<Tz: TimeZone>(&mut self, time: DateTime<Tz>, _12hr: bool) -> Zoom65Result<()> {

        // so the tkl dyna accepts two bytes for the year, convert the current year
        // so 2025 comes out as [00, 00, 07, e9] (hex), or [0, 0, 7, 233] (decimal)
        // if you can make this keyboard last beyond the year 65535, then i applaud you, because we've run
        // out bits to represent the date on this screen LOL
        let current_year: i32 = 2025;
        let current_year_first_hex: [u8; _] = current_year.to_be_bytes();
        let current_day = time.weekday().number_from_sunday() - 1;

        // let res: Vec<u8> = self.execute(abi::set_time(
        //     current_year_first_hex[2],
        //     current_year_first_hex[3],
        //     10 as u8,
        //     22 as u8,
        //     19 as u8,
        //     53 as u8,
        //     27 as u8,
        //     3 as u8
        // ))?;

        let res: Vec<u8> = self.execute(abi::generate_time_buffer(
            current_year_first_hex[2],
            current_year_first_hex[3],
            10 as u8,
            22 as u8,
            12 as u8,
            10 as u8,
            3 as u8,
            4 as u8
        ))?;

        if res[0] == 3 {
            let success_datapoint = [3 as u8];
            self.device.write(&success_datapoint);
            Ok(())
        } else {
            Err(Zoom65Error::UpdateCommandFailed)
        }
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

    // fn upload_media(
    //     &mut self,
    //     buf: impl AsRef<[u8]>,
    //     channel: UploadChannel,
    //     cb: impl Fn(usize),
    // ) -> Zoom65Result<()> {
    //     let image = buf.as_ref();

    //     // start upload
    //     let res = self.execute(abi::upload_start(channel))?;
    //     if res[1] != 1 || res[2] != 1 {
    //         return Err(Zoom65Error::UpdateCommandFailed);
    //     }
    //     let res = self.execute(abi::upload_length(image.len() as u32))?;
    //     if res[1] != 1 || res[2] != 1 {
    //         return Err(Zoom65Error::UpdateCommandFailed);
    //     }

    //     for (i, chunk) in image.chunks(24).enumerate() {
    //         cb(i);

    //         let chunk_len = chunk.len();
    //         let mut buf = [0u8; 32];

    //         // command prefix
    //         buf[0] = 0x0;
    //         buf[1] = 88;
    //         buf[2] = 2 + chunk_len as u8 + 4;

    //         // chunk index and data
    //         buf[3] = (i >> 8) as u8;
    //         buf[4] = (i & 255) as u8;
    //         buf[5..5 + chunk.len()].copy_from_slice(chunk);

    //         let mut offset = 3 + 2 + chunk_len;

    //         // Images are always aligned, but we need to manually align the last chunk of gifs
    //         if channel == UploadChannel::Gif && i == image.len() / 24 {
    //             // compute padding for final payload, the checksum needs 32-bit alignment
    //             let padding = (4 - (image.len() % 24) % 4) % 4;
    //             buf[2] += padding as u8;
    //             offset += padding;
    //         }

    //         // compute checksum
    //         let data = &buf[3..offset + 2];
    //         let crc = checksum(data);
    //         buf[offset..offset + 4].copy_from_slice(&crc);

    //         // send payload and read response
    //         let res = self.execute(buf)?;
    //         if res[1] != 1 || res[2] != 1 {
    //             return Err(Zoom65Error::UpdateCommandFailed);
    //         }
    //     }

    //     let res = self.execute(abi::upload_end())?;
    //     if res[1] != 1 || res[2] != 1 {
    //         return Err(Zoom65Error::UpdateCommandFailed);
    //     }

    //     // TODO: is this required?
    //     self.reset_screen()?;

    //     Ok(())
    // }

    // Upload an image to the keyboard. Must be encoded as 110x110 RGBA-3328 raw buffer
    // #[inline(always)]
    // pub fn upload_image(&mut self, buf: impl AsRef<[u8]>, cb: impl Fn(usize)) -> Zoom65Result<()> {
    //     let buf = buf.as_ref();
    //     if buf.len() != 36300 {
    //         return Err(Zoom65Error::GifTooLarge);
    //     }
    //     self.upload_media(buf, UploadChannel::Image, cb)
    // }

    // Upload a gif to the keyboard. Must be 111x111.
    // #[inline(always)]
    // pub fn upload_gif(&mut self, buf: impl AsRef<[u8]>, cb: impl Fn(usize)) -> Zoom65Result<()> {
    //     if buf.as_ref().len() >= 1013808 {
    //         return Err(Zoom65Error::GifTooLarge);
    //     }
    //     self.upload_media(buf, UploadChannel::Gif, cb)
    // }

    // Clear the image slot
    // #[inline(always)]
    // pub fn clear_image(&mut self) -> Zoom65Result<()> {
    //     let res = self.execute(abi::delete_image())?;
    //     (res[1] == 1 && res[2] == 1)
    //         .then_some(())
    //         .ok_or(Zoom65Error::UpdateCommandFailed)
    // }

    // Clear the gif slot
    // #[inline(always)]
    // pub fn clear_gif(&mut self) -> Zoom65Result<()> {
    //     let res = self.execute(abi::delete_gif())?;
    //     (res[1] == 1 && res[2] == 1)
    //         .then_some(())
    //         .ok_or(Zoom65Error::UpdateCommandFailed)
    // }
}
