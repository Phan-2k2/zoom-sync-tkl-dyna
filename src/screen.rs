use std::error::Error;

use bpaf::{Bpaf, Parser, construct};
use zoom65v3::Zoom65v3;
use zoom65v3::types::ScreenPosition;

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
            .req_flag(ScreenArgs::Reactive);
        construct!([reactive, screen_args()])
    }
}

pub fn apply_screen(args: &ScreenArgs, keyboard: &mut Zoom65v3) -> Result<(), Box<dyn Error>> {
    match args {
        ScreenArgs::Screen(pos) => keyboard.set_screen(*pos)?,
        ScreenArgs::Up => keyboard.screen_up()?,
        ScreenArgs::Down => keyboard.screen_down()?,
        ScreenArgs::Switch => keyboard.screen_switch()?,
        ScreenArgs::Reactive => todo!("cannot apply reactive gif natively"),
    };
    Ok(())
}
