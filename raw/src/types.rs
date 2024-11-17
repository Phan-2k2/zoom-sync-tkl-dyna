use hidapi::HidError;

#[derive(thiserror::Error)]
pub enum Zoom65Error {
    #[error("failed to find device")]
    DeviceNotFound,
    #[error("firmware version is unknown. open an issue for support")]
    UnknownFirmwareVersion,
    #[error("keyboard responded with error while updating, byte 1 == 88 && byte 2 == 0")]
    UpdateCommandFailed,
    #[error("{_0}")]
    Hid(#[from] HidError),
}

impl std::fmt::Debug for Zoom65Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Icon {
    DayClear = 0,
    DayPartlyCloudy = 1,
    DayPartlyRainy = 2,
    NightPartlyCloudy = 3,
    NightClear = 4,
    Cloudy = 5,
    Rainy = 6,
    Snowfall = 7,
    Thunderstorm = 8,
}

impl Icon {
    /// Convert a WMO index into a weather icon, adapting for day and night
    /// Adapted from the list at the bottom of <https://open-meteo.com/en/docs>
    pub fn from_wmo(wmo: u8, is_day: bool) -> Option<Self> {
        match wmo {
            // clear and mainly clear
            0 | 1 => Some(if is_day { Icon::DayClear } else { Icon::NightClear }),

            // partly cloudy
            2 => Some(if is_day { Icon::DayPartlyCloudy } else { Icon::NightPartlyCloudy }),

            // overcast
            3
            // foggy
            | 45 | 48 => Some(Icon::Cloudy),

            // drizzle
            51 | 53 | 55
            // freezing drizzle
            |56 | 57
            // rain
            | 61 | 63 | 65
            // freezing rain
            | 66 | 67 => Some(Icon::Rainy),

            // rain showers
            80..=82 => Some(if is_day { Icon::DayPartlyRainy } else { Icon::Rainy }),


            // snowfall
            | 71 | 73 | 75 | 77
            // snow showers
            | 85 | 86 => Some(Icon::Snowfall),

            // thunderstorm
            95 | 96 | 99 => Some(Icon::Thunderstorm),

            // unknown
            _ => None
        }
    }
}

