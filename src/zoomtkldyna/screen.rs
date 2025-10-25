use std::error::Error;

use bpaf::{Bpaf, Parser};
use crate::ZoomTklDyna;


/// Screen options:
#[derive(Clone, Copy, Debug, PartialEq, Eq, Bpaf)]
pub enum ScreenArgs {
    /// Move the screen up
    Up,
    /// Move the screen down
    Down,
    /// Switch the screen offset
    Enter,
    Return,
    Reset,
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

pub fn apply_screen(args: &ScreenArgs, keyboard: &mut ZoomTklDyna) -> Result<(), Box<dyn Error>> {
    match args {
        ScreenArgs::Up => keyboard.control_screen(ScreenArgs::Up)?,
        ScreenArgs::Down => keyboard.control_screen(ScreenArgs::Down)?,
        ScreenArgs::Enter => keyboard.control_screen(ScreenArgs::Enter)?,
        ScreenArgs::Return => keyboard.control_screen(ScreenArgs::Return)?,
        ScreenArgs::Reset => keyboard.control_screen(ScreenArgs::Reset)?,
        #[cfg(target_os = "linux")]
        ScreenArgs::Reactive => todo!("cannot apply reactive gif natively"),
    };
    Ok(())
}
