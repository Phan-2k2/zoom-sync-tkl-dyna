//! Main cli binary

use std::error::Error;
use std::io::Seek;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use bpaf::{Bpaf, Parser};
use either::Either;
use evdev::InputEventKind;
use futures::future::OptionFuture;
use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::AnimationDecoder;
use media::encode_gif;
use tokio_stream::StreamExt;
use zoom65v3::Zoom65v3;

use crate::info::{apply_system, cpu_mode, gpu_mode, system_args, CpuMode, GpuMode, SystemArgs};
use crate::media::encode_image;
use crate::screen::{apply_screen, screen_args, screen_args_with_reactive, ScreenArgs};
use crate::weather::{apply_weather, weather_args, WeatherArgs};

mod info;
mod media;
mod screen;
mod weather;

fn farenheit() -> impl Parser<bool> {
    bpaf::short('f')
        .long("farenheit")
        .help(
            "Use farenheit for all fetched temperatures. \
May cause clamping for anything greater than 99F.\
No effect on any manually provided data.",
        )
        .switch()
}

#[derive(Debug, Clone, Bpaf)]
struct RefreshArgs {
    /// Continuously refresh system data at a given interval (in seconds)
    #[bpaf(short('S'), long, fallback(10), display_fallback)]
    refresh_system: u64,
    /// Continuously refresh system data at a given interval (in seconds)
    #[bpaf(short('W'), long, fallback(60 * 60), display_fallback)]
    refresh_weather: u64,
    /// Retry interval for reconnecting to keyboard
    #[bpaf(short('R'), long, fallback(5), display_fallback)]
    retry: u64,
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
        /// Manually set download speed
        #[bpaf(short, long)]
        download: Option<f32>,
    },
    /// Change current screen
    #[bpaf(command, fallback_to_usage)]
    Screen(#[bpaf(external(screen_args))] ScreenArgs),
    /// Upload image/gif media
    #[bpaf(command, fallback_to_usage)]
    Image(
        /// Use nearest neighbor interpolation when resizing, otherwise uses gaussian.
        #[bpaf(short('n'), long("nearest"))] bool,
        /// Path to image to re-encode and upload
        #[bpaf(positional("PATH"))] PathBuf,
    ),
    /// Upload image/gif media
    #[bpaf(command, fallback_to_usage)]
    Gif(
        /// Use nearest neighbor interpolation when resizing, otherwise uses gaussian.
        #[bpaf(short('n'), long("nearest"))] bool,
        /// Path to animation to re-encode and upload
        #[bpaf(positional("PATH"))] PathBuf
    ),
    /// Clear media files
    #[bpaf(command)]
    Clear,
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version, descr(env!("CARGO_PKG_DESCRIPTION")))]
enum Cli {
    /// Update the keyboard periodically in a loop, reconnecting on errors.
    Run(#[bpaf(external(refresh_args))] RefreshArgs),
    /// Set specific options on the keyboard
    #[bpaf(command, fallback_to_usage)]
    Set {
        #[bpaf(external)]
        set_command: SetCommand,
    },
}

pub fn apply_time(keyboard: &mut Zoom65v3) -> Result<(), Box<dyn Error>> {
    let time = chrono::Local::now();
    keyboard
        .set_time(time)
        .map_err(|e| format!("failed to set time: {e}"))?;
    println!("updated time to {time}");
    Ok(())
}

async fn refresh(mut args: RefreshArgs) -> Result<(), Box<dyn Error>> {
    let mut cpu = match &args.system_args {
        SystemArgs::Disabled => None,
        SystemArgs::Enabled { cpu_mode, .. } => Some(cpu_mode.either()),
    };
    let gpu = match &args.system_args {
        SystemArgs::Disabled => None,
        SystemArgs::Enabled { gpu_mode, .. } => Some(gpu_mode.either()),
    };

    loop {
        if let Err(e) = run(&mut args, &mut cpu, &gpu).await {
            eprintln!("error: {e}\nreconnecting in {} seconds...", args.retry);
            tokio::time::sleep(Duration::from_secs(args.retry)).await;
        }
    }
}

async fn run(
    args: &mut RefreshArgs,
    cpu: &mut Option<Either<info::CpuTemp, u8>>,
    gpu: &Option<Either<info::GpuTemp, u8>>,
) -> Result<(), Box<dyn Error>> {
    let mut keyboard = Zoom65v3::open()?;
    let version = keyboard
        .get_version()
        .map_err(|e| format!("failed to get keyboard version: {e}"))?;
    println!("connected to keyboard version {version}");

    apply_time(&mut keyboard)?;

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

    #[cfg(not(target_os = "linux"))]
    let reactive_stream = None;
    #[cfg(target_os = "linux")]
    let mut reactive_stream = args.screen_args.and_then(|args| match args {
        ScreenArgs::Reactive => {
            println!("initializing reactive mode");
            keyboard
                .set_screen(zoom65v3::types::LogoOffset::Image.pos())
                .unwrap();
            let stream = evdev::enumerate().find_map(|(_, device)| {
                device
                    .name()
                    .unwrap()
                    .contains("Zoom65 v3 Keyboard")
                    .then_some(
                        device
                            .into_event_stream()
                            .map(|s| Box::pin(s.timeout(Duration::from_millis(500))))
                            .ok(),
                    )
                    .flatten()
            });
            if stream.is_none() {
                eprintln!("warning: couldn't find/access ev device");
            }
            stream
        },
        _ => None,
    });
    #[cfg(target_os = "linux")]
    let mut is_reactive_running = false;

    let mut weather_interval = tokio::time::interval(Duration::from_secs(args.refresh_weather));
    let mut system_interval = tokio::time::interval(Duration::from_secs(args.refresh_system));

    loop {
        tokio::select! {
            _ = weather_interval.tick() => {
                apply_weather(&mut keyboard, &mut args.weather_args, args.farenheit).await?
            },
            _ = system_interval.tick() => {
                if let SystemArgs::Enabled { download, .. } = args.system_args {
                    apply_system(
                        &mut keyboard,
                        args.farenheit,
                        cpu.as_mut().unwrap(),
                        gpu.as_ref().unwrap(),
                        download,
                    )?;
                }
            },
            Some(Some(res)) = OptionFuture::from(reactive_stream.as_mut().map(|s| s.next())) => {
                match res {
                    Ok(Err(e)) => return Err(Box::new(e)),
                    // keypress, play gif if not already running
                    Ok(Ok(ev)) if !is_reactive_running => {
                        if matches!(ev.kind(), InputEventKind::Key(_)) {
                            is_reactive_running = true;
                            keyboard.screen_switch()?;
                        }
                    },
                    // timeout, reset back to image
                    Err(_) if is_reactive_running => {
                        is_reactive_running = false;
                        keyboard.screen_switch()?;
                        keyboard.screen_switch()?;
                        keyboard.screen_switch()?;
                    },
                    _ => {}
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli().run();
    match args {
        Cli::Run(args) => refresh(args).await,
        Cli::Set { set_command } => {
            let mut keyboard = Zoom65v3::open()?;
            match set_command {
                SetCommand::Time => apply_time(&mut keyboard),
                SetCommand::Weather {
                    farenheit,
                    mut weather_args,
                } => apply_weather(&mut keyboard, &mut weather_args, farenheit).await,
                SetCommand::System {
                    farenheit,
                    cpu_mode,
                    gpu_mode,
                    download,
                } => apply_system(
                    &mut keyboard,
                    farenheit,
                    &mut cpu_mode.either(),
                    &gpu_mode.either(),
                    download,
                ),
                SetCommand::Screen(args) => apply_screen(&args, &mut keyboard),
                SetCommand::Image(nearest, path) => {
                    let image = ::image::open(path)?;
                    // re-encode and upload to keyboard
                    let encoded = encode_image(image, nearest).ok_or("failed to encode image")?;
                    keyboard.upload_image(encoded)?;
                    Ok(())
                },
                SetCommand::Gif(nearest, path) => {
                    println!("decoding animation...");
                    let decoder = image::ImageReader::open(path)?
                        .with_guessed_format()
                        .unwrap();
                    let frames = match decoder.format() {
                        Some(image::ImageFormat::Gif) => {
                            // Reset reader and decode gif as an animation
                            let mut reader = decoder.into_inner();
                            reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                            Some(GifDecoder::new(reader)?.into_frames())
                        },
                        Some(image::ImageFormat::Png) => {
                            // Reset reader
                            let mut reader = decoder.into_inner();
                            reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                            let decoder = PngDecoder::new(reader)?;
                            // If the png contains an apng, decode as an animation
                            decoder
                                .is_apng()?
                                .then_some(decoder.apng().unwrap().into_frames())
                        },
                        Some(image::ImageFormat::WebP) => {
                            // Reset reader
                            let mut reader = decoder.into_inner();
                            reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                            let decoder = WebPDecoder::new(reader).unwrap();
                            // If the webp contains an animation, decode as an animation
                            decoder.has_animation().then_some(decoder.into_frames())
                        },
                        _ => None,
                    }
                    .ok_or("failed to decode animation")?;

                    // re-encode and upload to keyboard
                    println!("re-encoding gif...");
                    let encoded = encode_gif(frames, nearest).ok_or("failed to encode gif image")?;
                    println!("encoded len: {}", encoded.len());
                    keyboard.upload_gif(encoded)?;
                    Ok(())
                },
                SetCommand::Clear => {
                    keyboard.clear_image()?;
                    keyboard.clear_gif()?;
                    Ok(())
                },
            }
        },
    }
}
