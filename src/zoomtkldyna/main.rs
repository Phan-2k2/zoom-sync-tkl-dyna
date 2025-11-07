// #![windows_subsystem = "windows"]
use std::error::Error;
use std::fmt::{Debug};
use std::thread::sleep;
use std::time::{Duration, Instant};

use bpaf::{Bpaf, Parser};
use either::Either;

use crate::info::{apply_system, cpu_mode, gpu_mode, system_args, CpuMode, GpuMode, SystemArgs};
use crate::screen::{apply_screen, screen_args, screen_args_with_reactive, ScreenArgs};
use crate::weather::{apply_weather, weather_args, WeatherArgs};

mod info;
mod media;
mod screen;
mod weather;
mod board_specific;
use board_specific::zoomtkldyna::ZoomTklDyna;

fn farenheit() -> impl Parser<bool> {
    bpaf::short('f')
        .long("farenheit")
        .help(
            "Use farenheit for all fetched temperatures. \
May cause clamping for anything greater than 99F. \
No effect on any manually provided data.",
        )
        .switch()
}

#[derive(Debug, Clone, Bpaf)]
struct RefreshArgs {
    /// Interval in seconds to refresh system data
    #[bpaf(short('S'), long, fallback(Duration::from_secs(10).into()), display_fallback)]
    refresh_system: humantime::Duration,
    /// Interval in seconds to refresh weather data
    #[bpaf(short('W'), long, fallback(Duration::from_secs(60 * 60).into()), display_fallback)]
    refresh_weather: humantime::Duration,
    /// Retry interval for reconnecting to keyboard
    #[bpaf(short('R'), long, fallback(Duration::from_secs(5).into()), display_fallback)]
    retry: humantime::Duration,
    /// Enable simulating 12hr time
    #[bpaf(long("12hr"), fallback(false), display_fallback)]
    _12hr: bool,
    #[bpaf(external)]
    farenheit: bool,
    #[bpaf(external(screen_args_with_reactive), optional)]
    screen_args: Option<ScreenArgs>,
    #[bpaf(external)]
    weather_args: WeatherArgs,
    #[bpaf(external)]
    system_args: SystemArgs,
}

#[derive(Clone, Debug, Bpaf)]
enum SetCommand {
    /// Sync time to system clock
    #[bpaf(command)]
    Time,
    /// Set weather data
    #[bpaf(command)]
    Weather {
        #[bpaf(external)]
        farenheit: bool,
        #[bpaf(external)]
        weather_args: WeatherArgs,
    },
    /// Set system info
    #[bpaf(command)]
    System {
        #[bpaf(external)]
        farenheit: bool,
        #[bpaf(external)]
        cpu_mode: CpuMode,
        #[bpaf(external)]
        gpu_mode: GpuMode,
        /// Manually set fan speed
        #[bpaf(short, long)]
        speed_fan: Option<u32>,
        /// Manually set download speed
        #[bpaf(short, long)]
        download: Option<f32>,
    },
    /// Change current screen
    #[bpaf(command, fallback_to_usage)]
    Screen(#[bpaf(external(screen_args))] ScreenArgs),
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version, descr(env!("CARGO_PKG_DESCRIPTION")))]
enum Cli {
    /// Update the keyboard periodically in a loop, reconnecting on errors.
    Run(#[bpaf(external(refresh_args))] RefreshArgs),
    /// Set specific options on the keyboard.
    /// Must not be used while zoom-sync is already running.
    #[bpaf(command, fallback_to_usage)]
    Set {
        #[bpaf(external)]
        set_command: SetCommand,
    },
}

pub fn apply_time(keyboard: &mut ZoomTklDyna, _12hr: bool) -> Result<(), Box<dyn Error>> {
    let time = chrono::Local::now();
    keyboard
        .set_time(time, _12hr)
        .map_err(|e| format!("failed to set time: {e}"))?;
    println!("updated time to {time}");
    Ok(())
}

fn refresh(mut args: RefreshArgs) -> Result<(), Box<dyn Error>> {
    let mut cpu = match &args.system_args {
        SystemArgs::Disabled => None,
        SystemArgs::Enabled { cpu_mode, .. } => Some(cpu_mode.either()),
    };
    let gpu = match &args.system_args {
        SystemArgs::Disabled => None,
        SystemArgs::Enabled { gpu_mode, .. } => Some(gpu_mode.either()),
    };

    loop {
        if let Err(e) = run(&mut args, &mut cpu, &gpu) {
            eprintln!("error: {e}\nreconnecting in {} seconds...", args.retry);
            sleep(args.retry.into());
        }
    }
}

fn run(
    args: &mut RefreshArgs,
    cpu: &mut Option<Either<info::CpuTemp, u8>>,
    gpu: &Option<Either<info::GpuStats, u32>>,
) -> Result<(), Box<dyn Error>> {
    let mut keyboard = ZoomTklDyna::open()?;
    println!("connected to keyboard");

    if let Some(ref args) = args.screen_args {
        #[cfg(not(target_os = "linux"))]
        {
            apply_screen(args, &mut keyboard)?;
            println!("set screen");
        }
        #[cfg(target_os = "linux")]
        if *args != ScreenArgs::Reactive {
            apply_screen(args, &mut keyboard)?;
            println!("set screen");
        }
    }
    // #[cfg(not(target_os = "linux"))]
    // let mut reactive_stream: Option<
    //     Box<
    //         dyn tokio_stream::Stream<Item = Result<Result<(), std::io::Error>, Box<dyn Error>>>
    //             + Unpin,
    //     >,
    // > = None;
    // #[cfg(target_os = "linux")]
    // let mut reactive_stream = args.screen_args.and_then(|args| match args {
    //     #[cfg(target_os = "linux")]
    //     ScreenArgs::Reactive => {
    //         println!("initializing reactive mode");
    //         keyboard
    //             .set_screen(zoomtkldyna::types::LogoOffset::Image.pos())
    //             .unwrap();
    //         let stream = evdev::enumerate().find_map(|(_, device)| {
    //             device
    //                 .name()
    //                 .unwrap()
    //                 .contains("Zoom TKL Dyna Keyboard")
    //                 .then_some(
    //                     device
    //                         .into_event_stream()
    //                         .map(|s| Box::pin(s.timeout(Duration::from_millis(500))))
    //                         .ok(),
    //                 )
    //                 .flatten()
    //         });
    //         if stream.is_none() {
    //             eprintln!("warning: couldn't find/access ev device");
    //         }
    //         stream
    //     },
    //     _ => None,
    // });
    // let mut is_reactive_running = false;

    // Sync time and weather immediately
    apply_time(&mut keyboard, args._12hr)?;
    apply_weather(&mut keyboard, &mut args.weather_args, args.farenheit)?;

    let weather_interval : Duration = args.refresh_weather.into();
    let system_interval : Duration = args.refresh_system.into();
    let loop_interval : Duration;
    if weather_interval > system_interval {
        loop_interval = system_interval;
    } else {
        loop_interval = weather_interval;
    }

    let mut weather_since_last_refresh = Instant::now();
    let mut system_since_last_refresh = Instant::now();
    let mut time_since_last_refresh = Instant::now();

    loop {
        if (time_since_last_refresh.elapsed() > Duration::from_secs(60)) && args._12hr {
            apply_time(&mut keyboard, args._12hr)?;
            time_since_last_refresh = Instant::now();
        }
        if weather_since_last_refresh.elapsed() > weather_interval {
            apply_weather(&mut keyboard, &mut args.weather_args, args.farenheit)?;
            weather_since_last_refresh = Instant::now();
        }
        if system_since_last_refresh.elapsed() > system_interval {
            if let SystemArgs::Enabled { speed_fan, download, .. } = args.system_args {
                apply_system(
                    &mut keyboard,
                    args.farenheit,
                    cpu.as_mut().unwrap(),
                    gpu.as_ref().unwrap(),
                    speed_fan,
                    download,
                )?;
            }
            system_since_last_refresh = Instant::now();
        }

        sleep(loop_interval);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli().run();
    match args {
        Cli::Run(args) => refresh(args),
        Cli::Set { set_command } => {
            let mut keyboard = ZoomTklDyna::open()?;
            match set_command {
                SetCommand::Time => apply_time(&mut keyboard, false),
                SetCommand::Weather {
                    farenheit,
                    mut weather_args,
                } => apply_weather(&mut keyboard, &mut weather_args, farenheit),
                SetCommand::System {
                    farenheit,
                    cpu_mode,
                    gpu_mode,
                    speed_fan,
                    download,
                } => apply_system(
                    &mut keyboard,
                    farenheit,
                    &mut cpu_mode.either(),
                    &gpu_mode.either(),
                    speed_fan,
                    download,
                ),
                SetCommand::Screen(args) => {apply_screen(&args, &mut keyboard)},
                // SetCommand::Image(args) => match args {
                //     SetMediaArgs::Set { nearest, path, bg } => {
                //         let image = ::image::open(path)?;
                //         // re-encode and upload to keyboard
                //         let encoded =
                //             encode_image(image, bg.0, nearest).ok_or("failed to encode image")?;
                //         let len = encoded.len();
                //         let total = len / 24;
                //         let width = total.to_string().len();
                //         keyboard.upload_image(encoded, |i| {
                //             print!("\ruploading {len} bytes ({i:width$}/{total}) ... ");
                //             stdout().flush().unwrap();
                //         })?;
                //         Ok(())
                //     },
                //     SetMediaArgs::Clear => {
                //         keyboard.clear_image()?;
                //         Ok(())
                //     },
                // },
                // SetCommand::Gif(args) => match args {
                //     SetMediaArgs::Set { nearest, path, bg } => {
                //         print!("decoding animation ... ");
                //         stdout().flush().unwrap();
                //         let decoder = image::ImageReader::open(path)?
                //             .with_guessed_format()
                //             .unwrap();
                //         let frames = match decoder.format() {
                //             Some(image::ImageFormat::Gif) => {
                //                 // Reset reader and decode gif as an animation
                //                 let mut reader = decoder.into_inner();
                //                 reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                //                 Some(GifDecoder::new(reader)?.into_frames())
                //             },
                //             Some(image::ImageFormat::Png) => {
                //                 // Reset reader
                //                 let mut reader = decoder.into_inner();
                //                 reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                //                 let decoder = PngDecoder::new(reader)?;
                //                 // If the png contains an apng, decode as an animation
                //                 decoder
                //                     .is_apng()?
                //                     .then_some(decoder.apng().unwrap().into_frames())
                //             },
                //             Some(image::ImageFormat::WebP) => {
                //                 // Reset reader
                //                 let mut reader = decoder.into_inner();
                //                 reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                //                 let decoder = WebPDecoder::new(reader).unwrap();
                //                 // If the webp contains an animation, decode as an animation
                //                 decoder.has_animation().then_some(decoder.into_frames())
                //             },
                //             _ => None,
                //         }
                //         .ok_or("failed to decode animation")?;
                //         println!("done");

                //         // re-encode and upload to keyboard
                //         let encoded = encode_gif(frames, bg.0, nearest)
                //             .ok_or("failed to encode gif image")?;
                //         let len = encoded.len();
                //         let total = len / 24;
                //         let width = total.to_string().len();
                //         keyboard.upload_gif(encoded, |i| {
                //             print!("\ruploading {len} bytes ({i:width$}/{total}) ... ");
                //             stdout().flush().unwrap();
                //         })?;
                //         println!("done");
                //         Ok(())
                //     },
                //     SetMediaArgs::Clear => {
                //         keyboard.clear_gif()?;
                //         Ok(())
                //     },
                // },
                // SetCommand::Clear => {
                //     keyboard.clear_image()?;
                //     keyboard.clear_gif()?;
                //     println!("cleared media");
                //     Ok(())
                // },
            }
        },
    }
}

#[cfg(test)]
#[test]
fn generate_docs() {
    let app = env!("CARGO_PKG_NAME");
    let options = cli();

    let roff = options.render_manpage(app, bpaf::doc::Section::General, None, None, None);
    std::fs::write("docs/zoom-sync.1", roff).expect("failed to write manpage");

    let md = options.header("").render_markdown(app);
    std::fs::write("docs/README.md", md).expect("failed to write markdown docs");
}
