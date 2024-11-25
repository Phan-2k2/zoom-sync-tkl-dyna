use crate::float::DumbFloat16;
use crate::types::{Icon, ScreenTheme, UploadChannel};

pub trait Arg {
    const SIZE: usize;
    fn to_bytes(&self) -> Vec<u8>;
}

impl Arg for u8 {
    const SIZE: usize = 1;
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl Arg for u32 {
    const SIZE: usize = 4;
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

macro_rules! impl_command_abi {
    [$(
        $( #[doc = $( $doc:tt )* ] )*
        const $cmd_name:ident = $cmd:expr;
        fn $name:ident ( $([ $( $hardcode:expr ),* ])? $( $arg:ident: $type:tt ),* );
    )+] => {
        $(
            $(#[doc = concat!("Prefix for ", $($doc)* )])*
            pub const $cmd_name: [u8; 3] = $cmd;

            $(#[doc = concat!("Construct a payload for ", $($doc)*)])*
            #[allow(unused_mut, unused_variables, unused_assignments)]
            pub fn $name( $( $arg: $type, )* ) -> [u8; 33] {
                let len = $cmd_name.len() $($( + $hardcode - $hardcode + 1 )*)? $( + $type::SIZE )*;
                let mut buf = [0u8; 33];
                buf[0] = 0x0;
                buf[1] = 88;
                buf[2] = len as u8;
                buf[3..6].copy_from_slice(&$cmd_name);
                let mut cur = 6;
                $($(
                    buf[cur] = $hardcode;
                    cur += 1;
                )*)?
                $(
                    let start = cur;
                    cur += $type::SIZE;
                    buf[start..cur].copy_from_slice(&$arg.to_bytes());
                )*
                buf
            }
        )*
    };
}

impl_command_abi![
    /* SCREEN POSITION */

    /// resetting screen back to meletrix logo
    const RESET_SCREEN = [165, 1, 255];
    fn reset_screen();

    const SCREEN_THEME = [165, 1, 255];
    fn screen_theme(theme: ScreenTheme);

    /// moving the screen up one position
    const SCREEN_UP = [165, 0, 34];
    fn screen_up();

    /// moving the screen down one position
    const SCREEN_DOWN = [165, 0, 33];
    fn screen_down();

    /// switching the screen to the next page
    const SCREEN_SWITCH = [165, 0, 32];
    fn screen_switch();

    /* MEDIA COMMANDS */

    /// deleting the currently uploaded image and reset back to the chrome dino
    const DELETE_IMAGE = [165, 2, 224];
    fn delete_image();

    /// deleting the currently uploaded gif and reset back to nyan cat
    const DELETE_GIF = [165, 2, 225];
    fn delete_gif();

    /// signaling the start of an upload
    const UPLOAD_START = [165, 2, 240];
    fn upload_start(channel: UploadChannel);

    /// signaling the length of an upload
    const UPLOAD_LENGTH = [165, 2, 208];
    fn upload_length(len: u32);

    /// signaling the end of an upload
    const UPLOAD_END = [165, 2, 241];
    fn upload_end([1]);

    /* SETTER COMMANDS */

    /// setting the system clock
    const SET_TIME = [165, 1, 16];
    fn set_time(year: u8, month: u8, day: u8, hour: u8, minute: u8, second: u8);

    /// setting the weather icon and current/min/max temperatures
    const SET_WEATHER = [165, 1, 32];
    fn set_weather(icon: Icon, current: u8, low: u8, high: u8);

    /// setting the cpu/gpu temp and download rate
    const SET_SYSTEM_INFO = [165, 1, 64];
    fn set_system_info(cpu_temp: u8, gpu_temp: u8, download: DumbFloat16);
];

/* GETTER COMMANDS */

/// Construct a payload for getting the abi version of the keyboard
pub const fn get_version() -> [u8; 33] {
    let mut buf = [0u8; 33];
    buf[1] = 1;
    buf
}
