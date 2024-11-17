//! Utilities for getting weather info

use std::error::Error;

use ipinfo::IpInfo;
use open_meteo_api::query::OpenMeteo;

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

pub async fn get_coords() -> Result<(f32, f32), Box<dyn Error>> {
    println!("fetching geolocation from ipinfo ...");
    let mut ipinfo = IpInfo::new(ipinfo::IpInfoConfig {
        token: None,
        ..Default::default()
    })?;
    let info = ipinfo.lookup_self_v4().await?;
    let (lat, long) = info.loc.split_once(',').unwrap();
    Ok((lat.parse().unwrap(), long.parse().unwrap()))
}

/// Get the current weather, using ipinfo for geolocation, and open-meteo for forcasting
pub async fn get_weather(
    lat: f32,
    long: f32,
    farenheit: bool,
) -> Result<(Icon, f32, f32, f32), Box<dyn Error>> {
    println!("fetching current weather from open-meteo for [{lat}, {long}] ...");
    let res = OpenMeteo::new()
        .coordinates(lat, long)?
        .current_weather()?
        .time_zone(open_meteo_api::models::TimeZone::Auto)?
        .daily()?
        .query()
        .await?;

    let current = res.current_weather.unwrap();
    let icon = Icon::from_wmo(current.weathercode as u8, current.is_day == 1.0).unwrap();

    let daily = res.daily.unwrap();
    let mut min = daily.temperature_2m_min.first().unwrap().unwrap();
    let mut max = daily.temperature_2m_max.first().unwrap().unwrap();
    let mut temp = current.temperature;

    // convert measurements to farenheit
    if farenheit {
        min = min * 9. / 5. + 32.;
        max = max * 9. / 5. + 32.;
        temp = temp * 9. / 5. + 32.;
    }

    Ok((icon, min, max, temp))
}
