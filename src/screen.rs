use std::error::Error;

use bpaf::Bpaf;
use zoom65v3::{types::ScreenPosition, Zoom65v3};

/// Screen options:
#[derive(Clone, Debug, Bpaf)]
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
}

pub fn apply_screen(args: &ScreenArgs, keyboard: &mut Zoom65v3) -> Result<(), Box<dyn Error>> {
    match args {
        ScreenArgs::Screen(pos) => keyboard.set_screen(*pos)?,
        ScreenArgs::Up => keyboard.screen_up()?,
        ScreenArgs::Down => keyboard.screen_down()?,
        ScreenArgs::Switch => keyboard.screen_switch()?,
    };
    Ok(())
}
