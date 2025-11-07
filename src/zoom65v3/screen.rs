use std::error::Error;

use bpaf::{Bpaf, Parser};
use crate::board_specific::types::ScreenPosition;
use crate::Zoom65v3;

/// Screen options:
#[derive(Clone, Copy, Debug, PartialEq, Eq, Bpaf)]
pub enum ScreenArgs {
    Screen(
        /// Reset and move the screen to a specific position.
        /// [cpu|gpu|download|time|weather|meletrix|zoom65|image|gif|battery]
        #[bpaf(short('s'), long("screen"), argument("POSITION"))]
        ScreenPosition,
    ),
    /// Move the screen up
    Up,
    /// Move the screen down
    Down,
    /// Switch the screen offset
    Switch,
    #[cfg(target_os = "linux")]
    /// Reactive image/gif mode
    #[bpaf(skip)]
    Reactive,
}

pub fn screen_args_with_reactive() -> impl Parser<ScreenArgs> {
    #[cfg(not(target_os = "linux"))]
    {
        screen_args()
    }

    #[cfg(target_os = "linux")]
    {
        let reactive = bpaf::long("reactive")
            .help("Enable reactive mode, playing gif when typing and image when resting. Requires root permission for reading keypresses via evdev")
            .req_flag(ScreenArgs::Reactive);
        bpaf::construct!([reactive, screen_args()]).group_help("Screen options:")
    }
}

pub fn apply_screen(args: &ScreenArgs, keyboard: &mut Zoom65v3) -> Result<(), Box<dyn Error>> {
    match args {
        ScreenArgs::Screen(pos) => keyboard.set_screen(*pos)?,
        ScreenArgs::Up => keyboard.screen_up()?,
        ScreenArgs::Down => keyboard.screen_down()?,
        ScreenArgs::Switch => keyboard.screen_switch()?,
        #[cfg(target_os = "linux")]
        ScreenArgs::Reactive => todo!("cannot apply reactive gif natively"),
    };
    Ok(())
}
