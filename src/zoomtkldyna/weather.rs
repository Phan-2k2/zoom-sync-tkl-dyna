//! Utilities for getting weather info

use std::error::Error;

use bpaf::Bpaf;
use chrono::Timelike;
use ipinfo::IpInfo;
use open_meteo_api::query::OpenMeteo;

use crate::board_specific::types::Icon;
use crate::ZoomTklDyna;

#[derive(Clone, Debug, Bpaf)]
#[bpaf(adjacent)]
pub struct Coords {
    /// Optional coordinates to use for fetching weather data, skipping ipinfo geolocation api.
    #[bpaf(long)]
    #[allow(dead_code)]
    pub coords: (),
    /// Latitude
    #[bpaf(positional("LAT"))]
    pub lat: f32,
    /// Longitude
    #[bpaf(positional("LON"))]
    pub long: f32,
}

/// Weather forecast options:
#[derive(Clone, Debug, Bpaf)]
pub enum WeatherArgs {
    /// Disable updating weather info completely
    #[bpaf(long("no-weather"))]
    Disabled,
    // default
    Auto {
        #[bpaf(external, optional)]
        coords: Option<Coords>,
    },
    #[bpaf(adjacent)]
    Manual {
        /// Manually provide weather data, skipping open-meteo weather api. All values are
        /// unitless.
        #[bpaf(short, long)]
        #[allow(dead_code)]
        weather: (),
        /// WMO Index
        #[bpaf(positional("WMO"))]
        wmo: i32,
        /// Current temperature
        #[bpaf(positional("CUR"))]
        current: f32,
        /// Minumum temperature
        #[bpaf(positional("MIN"))]
        min: f32,
        /// Maximum temperature
        #[bpaf(positional("MAX"))]
        max: f32,
    },
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
    let icon = Icon::from_wmo(current.weathercode as i32, current.is_day == 1.0).unwrap();

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

#[tokio::main]
pub async fn apply_weather(
    keyboard: &mut ZoomTklDyna,
    args: &mut WeatherArgs,
    farenheit: bool,
) -> Result<(), Box<dyn Error>> {
    match args {
        WeatherArgs::Disabled => println!("skipping weather"),
        WeatherArgs::Auto { coords } => {
            // attempt to backfill coordinates if not provided
            if coords.is_none() {
                match get_coords().await {
                    Ok((lat, long)) => {
                        *coords = Some(Coords {
                            coords: (),
                            lat,
                            long,
                        })
                    },
                    Err(e) => eprintln!("warning: failed to fetch geolocation from ipinfo: {e}"),
                }
            }

            // try to update weather if we have some coordinates
            if let Some(Coords { lat, long, .. }) = *coords {
                match get_weather(lat, long, farenheit).await {
                    Ok((icon, min, max, temp)) => {
                        keyboard
                            .set_weather(icon.clone(), temp, max, min)
                            .map_err(|e| format!("failed to set weather: {e}"))?;
                        println!(
                            "updated weather {{ icon: {icon:?}, current: {temp}, min: {min}, max: {max} }}"
                        );
                    },
                    Err(e) => eprintln!("failed to fetch weather, skipping: {e}"),
                }
            }
        },
        WeatherArgs::Manual {
            wmo,
            current,
            min,
            max,
            ..
        } => {
            let hour = chrono::Local::now().hour();
            let is_day = (6..=18).contains(&hour);
            keyboard.set_weather(
                Icon::from_wmo(*wmo, is_day).ok_or("unknown WMO code")?,
                *current,
                *min,
                *max,
            )?;
        },
    }

    Ok(())
}
