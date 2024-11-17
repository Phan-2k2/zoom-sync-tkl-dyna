//! Utilities for getting weather info

use std::error::Error;

use ipinfo::IpInfo;
use open_meteo_api::query::OpenMeteo;
use zoom_sync_raw::types::Icon;

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
