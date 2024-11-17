//! Main cli binary

use std::{error::Error, time::Duration};
use zoom_sync_raw::Zoom65v3;

mod info;
mod weather;

#[derive(Clone, Debug, bpaf::Bpaf)]
#[bpaf(options, version, max_width(80), descr(env!("CARGO_PKG_DESCRIPTION")))]
struct Cli {
    /// Refresh data every given number of seconds
    #[bpaf(short, long, argument("SECS"), fallback(30), display_fallback)]
    refresh: u64,
    /// Use farenheit for all temperatures. May cause clamping for anything greater than 99F
    #[bpaf(short, long, fallback(false), display_fallback)]
    farenheit: bool,
    /// Use a specific gpu device id
    #[bpaf(short, long, argument("ID"), fallback(0), display_fallback)]
    gpu: u32,
    /// Search for a specific cpu temp component
    #[bpaf(short, long, argument("LABEL"), fallback("coretemp Package".into()), display_fallback)]
    temp: String,
    /// Optional coordinates to use for open-meteo weather forcasting.
    /// If unset, falls back to ipinfo.com for location.
    #[bpaf(short, long, argument::<String>("LAT,LONG"), parse(|v| {
        let (la, lo) = v.split_once(',').ok_or("coordinates should be lat,long")?;
        let la: f32 = la.parse().map_err(|_| "invalid latitude")?;
        let lo: f32 = lo.parse().map_err(|_| "invalid longitude")?;
        Result::<_, &str>::Ok((la, lo))
    }), optional)]
    coords: Option<(f32, f32)>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = cli().run();

    let mut keyboard = Zoom65v3::open()?;
    let version = keyboard
        .get_version()
        .map_err(|e| format!("failed to get keyboard version: {e}"))?;
    println!("connected to keyboard version {version}\n");

    let mut cpu = info::CpuTemp::new(&args.temp);
    let gpu = info::GpuTemp::new(args.gpu);

    loop {
        // update time
        let time = chrono::Local::now();
        keyboard
            .set_time(time)
            .map_err(|e| format!("failed to set time: {e}"))?;
        println!("updated time to {time}");

        // update system info
        let mut cpu_temp = cpu.get_temp(args.farenheit).unwrap_or_default();
        if cpu_temp >= 100 {
            eprintln!("warning: actual cpu temperature at {cpu_temp}, clamping to 99");
            cpu_temp = 99;
        }
        let mut gpu_temp = gpu.get_temp(args.farenheit).unwrap_or_default();
        if gpu_temp >= 100 {
            eprintln!("warning: actual gpu temerature at {gpu_temp}. clamping to 99");
            gpu_temp = 99;
        }
        keyboard
            .set_system_info(
                cpu_temp, gpu_temp, // TODO: fetch download
                0.,
            )
            .map_err(|e| format!("failed to set system info: {e}"))?;
        println!(
            "updated system info {{ cpu_temp: {cpu_temp}, gpu_temp: {gpu_temp}, download: 0.0 }}"
        );

        // Attempt to backfill coordinates from ipinfo if not provided.
        // Will keep retrying on each iteration until they are found.
        if args.coords.is_none() {
            match weather::get_coords().await {
                Ok(v) => args.coords = Some(v),
                Err(e) => eprintln!("warning: failed to fetch coordinates from ipinfo: {e}"),
            }
        }

        // try to update weather if we have some coordinates
        if let Some((lat, long)) = args.coords {
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

            println!("waiting {}s\n", args.refresh);
            std::thread::sleep(Duration::from_secs(args.refresh));
        }
    }
}
