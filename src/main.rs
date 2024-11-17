mod info;
mod weather;
mod zoom65;

#[derive(Clone, Debug, bpaf::Bpaf)]
#[bpaf(options)]
struct Cli {
    /// Use celcius for all temps
    #[bpaf(short, long, fallback(false), display_fallback)]
    farenheit: bool,
    /// Use a specific gpu device id
    #[bpaf(short, long, argument("ID"), fallback(0), display_fallback)]
    gpu: u32,
    /// Search for a specific cpu temp component
    #[bpaf(short, long, argument("LABEL"), fallback("coretemp Package id 0".into()), display_fallback)]
    temp: String,
    /// Optional geocoordinates to use for open-meteo weather forcasting
    #[bpaf(short, long, argument::<String>("LAT,LONG"), parse(|v| {
        let (la, lo) = v.split_once(',').ok_or("coordinates should be lat,long")?;
        let la: f32 = la.parse().map_err(|_| "invalid latitude")?;
        let lo: f32 = lo.parse().map_err(|_| "invalid longitude")?;
        Result::<_, &str>::Ok((la, lo))
    }), optional)]
    coords: Option<(f32, f32)>
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = cli().run();

    let mut keyboard =
        zoom65::Zoom65v3::open().map_err(|e| format!("failed to open device: {e}"))?;

    let version = keyboard
        .get_version()
        .map_err(|e| format!("failed to get keyboard version: {e}"))?;
    println!("keyboard version: {version}",);

    // setup time
    let time = chrono::Local::now();
    keyboard
        .set_time(time)
        .map_err(|e| format!("failed to set time: {e}"))?;
    println!("updated time to {time}");

    // setup weather
    let (icon, min, max, temp) = weather::get_weather(args.coords, args.farenheit).await;
    keyboard
        .set_weather(icon, temp as u8, max as u8, min as u8)
        .map_err(|e| format!("failed to set weather: {e}"))?;
    println!("updated weather {{ current: {temp}, min: {min}, max: {max} }}");

    // setup cpu/gpu temps
    let mut cpu = info::CpuTemp::new(Some(&args.temp));
    let gpu = info::GpuTemp::new(Some(args.gpu));
    keyboard
        .set_system_info(
            cpu.get_temp(args.farenheit).unwrap_or_default().clamp(0, 99),
            gpu.get_temp(args.farenheit).unwrap_or_default().clamp(0, 99),
            // TODO: fetch download
            0.,
        )
        .map_err(|e| format!("failed to set system info: {e}"))?;
    println!("updated system info");

    Ok(())
}
