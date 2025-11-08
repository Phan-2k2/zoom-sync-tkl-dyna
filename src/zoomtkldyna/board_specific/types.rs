// use std::str::FromStr;

use hidapi::HidError;

use crate::board_specific::abi::Arg;

pub type ZoomTklDynaResult<T> = Result<T, ZoomTklDynaError>;

#[derive(thiserror::Error)]
pub enum ZoomTklDynaError {
    #[error("failed to find device")]
    DeviceNotFound,
    #[error("firmware version is unknown. open an issue for support")]
    UnknownFirmwareVersion,
    #[error("keyboard responded with error while updating")]
    UpdateCommandFailed,
    #[error("{_0}")]
    Hid(#[from] HidError),
}

impl std::fmt::Debug for ZoomTklDynaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Icon {
    DayClearSunny = 1,
    DayPartlyCloudy = 2,
    Cloudy = 3,
    Rainy = 4,
    Snowy = 5,
    NightClearMoon = 6,
    NightCloudy = 7,
    ErrorIcon = 8,
}

impl Icon {
    /// Convert a WMO index into a weather icon, adapting for day and night
    /// Adapted from the list at the bottom of <https://open-meteo.com/en/docs>
    pub fn from_wmo(wmo: i32, is_day: bool) -> Option<Self> {
        match wmo {
            // clear and mainly clear
            0 | 1 => Some(if is_day { Icon::DayClearSunny } else { Icon::NightClearMoon }),

            // partly cloudy
            2 => Some(Icon::Cloudy),

            // overcast
            3 => Some(Icon::Cloudy),
            // foggy
            45 => Some(Icon::Cloudy),
            48 => Some(Icon::Cloudy),

            // drizzle
            51 | 53 | 55 => Some(Icon::Rainy),
            // freezing drizzle
            56 => Some(Icon::Rainy),
            57 => Some(Icon::Rainy),
            // rain
            61 => Some(Icon::Rainy), 
            63 => Some(Icon::Rainy),
            65 => Some(Icon::Rainy),
            // freezing rain
            66 => Some(Icon::Rainy),
            67 => Some(Icon::Rainy),

            // snowfall
            71 => Some(Icon::Snowy),
            73 => Some(Icon::Snowy),
            75 => Some(Icon::Snowy),
            77 => Some(Icon::Snowy),
            
            // rain showers
            80 => Some(Icon::Rainy),
            81 | 82=> Some(Icon::Rainy),
            // snow showers
            85 => Some(Icon::Snowy),
            86 => Some(Icon::Snowy),

            // thunderstorm
            95 => Some(Icon::Rainy),
            96 | 99 => Some(Icon::Snowy),

            // unknown
            _ => Some(Icon::ErrorIcon)
        }
    }
}

impl Arg for Icon {
    const SIZE: usize = 1;
    #[inline(always)]
    fn to_bytes(&self) -> Vec<u8> {
        vec![self.clone() as u8]
    }
}
