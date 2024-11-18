//! Main cli binary

use bpaf::Bpaf;
use chrono::Timelike;
use either::Either;
use info::{CpuTemp, GpuTemp};
use std::{error::Error, time::Duration};
use zoom_sync_raw::{types::Icon, Zoom65v3};

mod info;
mod weather;

#[derive(Clone, Debug, Bpaf)]
#[bpaf(adjacent)]
struct Coords {
    /// Optional coordinates to use for fetching weather data, skipping ipinfo geolocation api.
    #[bpaf(long)]
    #[allow(dead_code)]
    coords: (),
    /// Latitude
    #[bpaf(positional("LAT"))]
    lat: f32,
    // Longitude
    #[bpaf(positional("LON"))]
    long: f32,
}

/// Weather forecast options:
#[derive(Clone, Debug, Bpaf)]
enum WeatherArgs {
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
        /// Manually provide weather data, skipping open-meteo weather api. All values are unitless.
        #[bpaf(short, long)]
        #[allow(dead_code)]
        weather: (),
        /// WMO Index
        #[bpaf(positional("WMO"))]
        wmo: u8,
        /// Current temperature
        #[bpaf(positional("CUR"))]
        current: u8,
        /// Minumum temperature
        #[bpaf(positional("MIN"))]
        min: u8,
        /// Maximum temperature
        #[bpaf(positional("MAX"))]
        max: u8,
    },
}

#[derive(Clone, Debug, bpaf::Bpaf)]
enum CpuMode {
    Label(
        /// Sensor label to search for
        #[bpaf(long("cpu"), argument("LABEL"), fallback("coretemp Package".into()), display_fallback)]
        String,
    ),
    Manual(
        /// Manually set CPU temperature
        #[bpaf(short('c'), long("cpu-temp"), argument("TEMP"))]
        u8,
    ),
}

#[derive(Clone, Debug, bpaf::Bpaf)]
enum GpuMode {
    Id(
        /// GPU device id to fetch temperature data for (nvidia only)
        #[bpaf(long("gpu"), argument::<u32>("ID"), fallback(0), display_fallback)]
        u32,
    ),
    Manual(
        /// Manually set GPU temperature
        #[bpaf(short('g'), long("gpu-temp"), argument("TEMP"))]
        u8,
    ),
}

/// System info options:
#[derive(Clone, Debug, Bpaf)]
enum SystemArgs {
    /// Disable updating system info completely
    #[bpaf(long("no-system"))]
    Disabled,
    Enabled {
        #[bpaf(external)]
        cpu_mode: CpuMode,
        #[bpaf(external)]
        gpu_mode: GpuMode,
        /// Manually set download speed
        #[bpaf(short, long)]
        download: Option<f64>,
    },
}

/// Sync modes:
#[derive(Clone, Debug, Bpaf)]
enum Mode {
    /// Update the keyboard a single time
    #[bpaf(short, long)]
    Update,
    Refresh(
        /// Continuously refresh the data at a given interval
        #[bpaf(short('r'), long("refresh"), fallback(30), display_fallback)]
        u64,
        /// Retry interval for reconnecting to keyboard
        #[bpaf(short('R'), long("retry"), fallback(5), display_fallback)]
        u64,
    ),
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version, descr(env!("CARGO_PKG_DESCRIPTION")))]
struct Cli {
    #[bpaf(external)]
    mode: Mode,
    /// Use farenheit for all fetched temperatures. May cause clamping for anything greater than 99F.
    /// No effect on any manually provided data.
    #[bpaf(short, long)]
    farenheit: bool,
    #[bpaf(external)]
    weather_args: WeatherArgs,
    #[bpaf(external)]
    system_args: SystemArgs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = cli().run();

    let mut cpu = match &args.system_args {
        SystemArgs::Enabled {
            cpu_mode: CpuMode::Label(label),
            ..
        } => Either::Left(CpuTemp::new(label)),
        SystemArgs::Enabled {
            cpu_mode: CpuMode::Manual(v),
            ..
        } => Either::Right(*v),
        SystemArgs::Disabled => Either::Right(0),
    };

    let gpu = match &args.system_args {
        SystemArgs::Enabled {
            gpu_mode: GpuMode::Id(id),
            ..
        } => Either::Left(GpuTemp::new(*id)),
        SystemArgs::Enabled {
            gpu_mode: GpuMode::Manual(v),
            ..
        } => Either::Right(*v),
        SystemArgs::Disabled => Either::Right(0),
    };

    match args.mode {
        Mode::Update => {
            let mut keyboard = Zoom65v3::open()?;
            run(&mut args, &mut keyboard, &mut cpu, &gpu).await?;
            std::process::exit(0);
        }
        Mode::Refresh(s, r) => 'outer: loop {
            let mut keyboard = match Zoom65v3::open() {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("error: {e}\nreconnecting in {r} seconds...");
                    tokio::time::sleep(Duration::from_secs(r)).await;
                    continue 'outer;
                }
            };

            let version = keyboard
                .get_version()
                .map_err(|e| format!("failed to get keyboard version: {e}"))?;
            println!("connected to keyboard version {version}");

            loop {
                println!();
                match run(&mut args, &mut keyboard, &mut cpu, &gpu).await {
                    Ok(_) => tokio::time::sleep(Duration::from_secs(s)).await,
                    Err(e) => {
                        eprintln!("error: {e}");
                        continue 'outer;
                    }
                }
            }
        },
    }
}

async fn run(
    args: &mut Cli,
    keyboard: &mut Zoom65v3,
    cpu: &mut Either<info::CpuTemp, u8>,
    gpu: &Either<info::GpuTemp, u8>,
) -> Result<(), Box<dyn Error>> {
    let time = chrono::Local::now();
    keyboard
        .set_time(time)
        .map_err(|e| format!("failed to set time: {e}"))?;
    println!("updated time to {time}");

    match args.system_args {
        SystemArgs::Disabled => println!("skipping system info"),
        SystemArgs::Enabled { download, .. } => {
            let mut cpu_temp = cpu
                .as_mut()
                .map_left(|c| c.get_temp(args.farenheit).unwrap_or_default())
                .map_right(|v| *v)
                .into_inner();
            if cpu_temp >= 100 {
                eprintln!("warning: actual cpu temperature at {cpu_temp}, clamping to 99");
                cpu_temp = 99;
            }

            let mut gpu_temp = gpu
                .as_ref()
                .map_left(|g| g.get_temp(args.farenheit).unwrap_or_default())
                .map_right(|v| *v)
                .into_inner();
            if gpu_temp >= 100 {
                eprintln!("warning: actual gpu temerature at {gpu_temp}. clamping to 99");
                gpu_temp = 99;
            }

            keyboard
                .set_system_info(cpu_temp, gpu_temp, download.unwrap_or_default())
                .map_err(|e| format!("failed to set system info: {e}"))?;
            println!("updated system info {{ cpu_temp: {cpu_temp}, gpu_temp: {gpu_temp}, download: 0.0 }}");
        }
    }

    match &mut args.weather_args {
        WeatherArgs::Disabled => println!("skipping weather"),
        WeatherArgs::Auto { coords } => {
            // attempt to backfill coordinates if not provided
            if coords.is_none() {
                match weather::get_coords().await {
                    Ok((lat, long)) => {
                        *coords = Some(Coords {
                            coords: (),
                            lat,
                            long,
                        })
                    }
                    Err(e) => eprintln!("warning: failed to fetch geolocation from ipinfo: {e}"),
                }
            }

            // try to update weather if we have some coordinates
            if let Some(Coords { lat, long, .. }) = *coords {
                match weather::get_weather(lat, long, args.farenheit).await {
                    Ok((icon, min, max, temp)) => {
                        keyboard
                            .set_weather(icon.clone(), temp as u8, max as u8, min as u8)
                            .map_err(|e| format!("failed to set weather: {e}"))?;
                        println!(
                            "updated weather {{ icon: {icon:?}, current: {temp}, min: {min}, max: {max} }}"
                        );
                    }
                    Err(e) => eprintln!("failed to fetch weather, skipping: {e}"),
                }
            }
        }
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
        }
    }

    Ok(())
}
