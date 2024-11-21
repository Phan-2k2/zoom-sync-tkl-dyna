//! Main cli binary

use std::path::PathBuf;
use std::{error::Error, time::Duration};

use bpaf::{Bpaf, Parser};
use either::Either;
use info::apply_system;
use weather::apply_weather;
use zoom65v3::Zoom65v3;

use crate::info::{cpu_mode, gpu_mode, system_args, CpuMode, GpuMode, SystemArgs};
use crate::screen::{apply_screen, screen_args, ScreenArgs};
use crate::weather::{weather_args, WeatherArgs};

mod info;
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
    /// Continuously refresh the data at a given interval
    #[bpaf(short, long, fallback(30), display_fallback)]
    refresh: u64,
    /// Retry interval for reconnecting to keyboard
    #[bpaf(short('R'), long, fallback(5), display_fallback)]
    retry: u64,
    #[bpaf(external)]
    farenheit: bool,
    #[bpaf(external, optional)]
    screen_args: Option<ScreenArgs>,
    #[bpaf(external(weather_args))]
    weather_args: WeatherArgs,
    #[bpaf(external(system_args))]
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
    #[bpaf(command)]
    Gif(#[bpaf(positional("PATH"))] PathBuf)
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

async fn refresh(mut args: RefreshArgs) -> Result<(), Box<dyn Error>> {
    let mut cpu = match &args.system_args {
        SystemArgs::Disabled => None,
        SystemArgs::Enabled { cpu_mode, .. } => Some(cpu_mode.either()),
    };
    let gpu = match &args.system_args {
        SystemArgs::Disabled => None,
        SystemArgs::Enabled { gpu_mode, .. } => Some(gpu_mode.either()),
    };

    'outer: loop {
        let mut keyboard = match Zoom65v3::open() {
            Ok(k) => k,
            Err(e) => {
                eprintln!("error: {e}\nreconnecting in {} seconds...", args.retry);
                tokio::time::sleep(Duration::from_secs(args.retry)).await;
                continue 'outer;
            }
        };

        let version = keyboard
            .get_version()
            .map_err(|e| format!("failed to get keyboard version: {e}"))?;
        println!("connected to keyboard version {version}");

        if let Some(ref args) = args.screen_args {
            if let Err(e) = apply_screen(args, &mut keyboard) {
                eprintln!("error: {e}");
                continue 'outer;
            }
            println!("set screen");
        }

        loop {
            println!();
            if let Err(e) = run(&mut args, &mut keyboard, &mut cpu, &gpu).await {
                eprintln!("error: {e}");
                continue 'outer;
            }
            tokio::time::sleep(Duration::from_secs(args.refresh)).await
        }
    }
}

async fn run(
    args: &mut RefreshArgs,
    keyboard: &mut Zoom65v3,
    cpu: &mut Option<Either<info::CpuTemp, u8>>,
    gpu: &Option<Either<info::GpuTemp, u8>>,
) -> Result<(), Box<dyn Error>> {
    apply_time(keyboard)?;
    if let SystemArgs::Enabled { download, .. } = args.system_args {
        apply_system(
            keyboard,
            args.farenheit,
            cpu.as_mut().unwrap(),
            gpu.as_ref().unwrap(),
            download,
        )?;
    }
    apply_weather(keyboard, &mut args.weather_args, args.farenheit).await?;

    Ok(())
}

pub fn apply_time(keyboard: &mut Zoom65v3) -> Result<(), Box<dyn Error>> {
    let time = chrono::Local::now();
    keyboard
        .set_time(time)
        .map_err(|e| format!("failed to set time: {e}"))?;
    println!("updated time to {time}");
    Ok(())
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
                SetCommand::Gif(path) => {
                    let gif = std::fs::read(path)?;
                    keyboard.upload_gif(gif)?;
                    Ok(())
                }
            }
        }
    }
}
