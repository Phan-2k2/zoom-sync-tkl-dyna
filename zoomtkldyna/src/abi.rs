use crate::float::DumbFloat16;
// use crate::types::{Icon, ScreenTheme, UploadChannel};
use crate::types::{Icon, ScreenTheme};

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

// I'm new to rust, this is some dark magic if i've ever seen one
macro_rules! impl_command_abi {
    [$(
        $( #[doc = $( $doc:tt )* ] )*
        fn $name:ident ( $([ $( $hardcode:expr ),* ]$(,)?)? $( $arg:ident: $type:tt ),* );
    )+] => {
        $(
            $(#[doc = concat!("Construct a payload for ", $($doc)*)])*
            #[allow(unused_mut, unused_variables, unused_assignments)]
            pub fn $name( $( $arg: $type ),* ) -> [u8; 33] {
                let len = const { 0 $($( + $hardcode - $hardcode + 1 )*)? $( + $type::SIZE )* };
                let mut buf = [0u8; 33];
                buf[0] = 0x0;
                buf[1] = 88;
                buf[2] = len as u8;
                let mut cur = 3;
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
    fn reset_screen([165, 1, 255]);

    /// set the screen theme
    fn screen_theme([165, 1, 255], theme: ScreenTheme);

    /// moving the screen up one position
    fn screen_up([165, 0, 34]);

    /// moving the screen down one position
    fn screen_down([165, 0, 33]);

    /// switching the screen to the next page
    fn screen_switch([165, 0, 32]);

    /* MEDIA COMMANDS */

    /// deleting the currently uploaded image and reset back to the chrome dino
    // fn delete_image([165, 2, 224]);

    /// deleting the currently uploaded gif and reset back to nyan cat
    // fn delete_gif([165, 2, 225]);

    /// signaling the start of an upload
    // fn upload_start([165, 2, 240], channel: UploadChannel);

    /// signaling the length of an upload
    // fn upload_length([165, 2, 208], len: u32);

    /// signaling the end of an upload
    // fn upload_end([165, 2, 241, 1]);

    /* SETTER COMMANDS */

    /// setting the cpu/gpu temp and download rate
    fn set_system_info([165, 1, 64], cpu_temp: u8, gpu_temp: u8, download: DumbFloat16);
];

//trying to replicate this bs as best I can because wtf
pub fn generate_time_buffer(year_byte1: u8, year_byte2: u8, month: u8, day: u8, hour: u8, minute: u8, second: u8, week_day: u8) -> [u8; 33] {

    let mut data_buffer:[u8; 32] = [0u8; 32];
    let data_buffer_len: usize = 32;
    let intial_data: [u8; 14] = [165, 56, 0, 10, 0, 1, year_byte1, year_byte2, month, day, hour, minute, second, week_day];
    data_buffer[8..22].copy_from_slice(&intial_data);

    //here lies some dumbass math they thought would be funny to include 
    //because why not obsfucate your code as much as you can
    //I'm literally calling them magic numbers because they're dumb AF
    let mut magic_number_1: i32 = 0;
    for x in 9..data_buffer_len {
        magic_number_1 += data_buffer[x] as i32
    }
    magic_number_1 = magic_number_1 ^ 255;
    data_buffer[22] = (magic_number_1 & 255) as u8;

    data_buffer[0] = 28;
    data_buffer[1] = 3;
    data_buffer[5] = 15;

    let magic_number_2: i32 = magic_checksum_processing(data_buffer, data_buffer_len);

    data_buffer[7] = (magic_number_2 >> 8) as u8;
    data_buffer[6] = magic_number_2 as u8;

    let mut final_buffer: [u8; 33] = [0u8; 33];
    final_buffer[0] = 0x0;
    final_buffer[1..33].copy_from_slice(&data_buffer);
    final_buffer

}

pub fn generate_weather_buffer(icon: Icon, current: f32, low: f32, high: f32) -> [u8; 33]  {
    // ke = current
    // xe = magic_temp_current_1
    // z = low
    // A = magic_temp_low_1
    // O = high
    // q = magic_temp_high_1

    let magic_temp_current_1 = magic_temp_processing(current);

    let magic_temp_low_1 = magic_temp_processing(low);

    let magic_temp_high_1 = magic_temp_processing(high);

    // let time = chrono::Local::now();
    // let cur_hours = time.day();

    //J = temp_data_array
    let mut temp_data_array: [u8; 9] = [0; 9];
    let temp_data_array_len: usize = temp_data_array.len();
    temp_data_array[0] = 254;
    temp_data_array[1] = 0;
    temp_data_array[2] = icon as u8;
    temp_data_array[3] = ((magic_temp_current_1 >> 8) & 255) as u8;
    temp_data_array[4] = magic_temp_current_1 as u8;
    temp_data_array[5] = ((magic_temp_low_1 >> 8) & 255) as u8;
    temp_data_array[6] = magic_temp_low_1 as u8;
    temp_data_array[7] = ((magic_temp_high_1 >> 8) & 255) as u8;
    temp_data_array[8] = magic_temp_high_1 as u8;

    //Pe = data_buffer
    let mut data_buffer = [0; 32];
    let data_buffer_len = data_buffer.len();
    data_buffer[8] = 165;
    data_buffer[9] = temp_data_array[0];
    data_buffer[10] = 0;
    data_buffer[11] = 8;
    
    for z in 1..temp_data_array_len {
        data_buffer[11 + z] = temp_data_array[z];
    }

    let mut magic_number_1: i32 = 0;
    for z in 9..data_buffer.len() {
        magic_number_1 += data_buffer[z] as i32
    }
    magic_number_1 = magic_number_1 ^ 255;

    data_buffer[12 + 8] = (magic_number_1 & 255) as u8;
    data_buffer[0] = 28;
    data_buffer[1] = 2;
    data_buffer[5] = 4 + 8 + 1;

    let magic_number_2 = magic_checksum_processing(data_buffer, data_buffer_len);

    data_buffer[7] = (magic_number_2 >> 8) as u8;
    data_buffer[6] = magic_number_2 as u8;

    let mut final_buffer: [u8; 33] = [0u8; 33];
    final_buffer[0] = 0x0;
    final_buffer[1..33].copy_from_slice(&data_buffer);
    final_buffer
}

pub fn generate_sysinfo_buffer(cpu_temp: u8, gpu_temp: u8, download: DumbFloat16) -> [u8; 33] {
    let mut data_buffer = [0; 32];
    let data_buffer_len = data_buffer.len();
    data_buffer[8] = 165;
    data_buffer[9] = 255;
    data_buffer[10] = 0;
    data_buffer[11] = 11;
    data_buffer[14] = cpu_temp;
    data_buffer[16] = gpu_temp;
    data_buffer[18] = 61; //or 61? I think this is SSD temp
    data_buffer[19] = 2; // This is fan rpm, 1 = 256, 2 = 512, the next one adds on.
    data_buffer[20] = 0; // Fan RPM modifier, +1 to above.
    data_buffer[21] = 2; // This is network speed, 1 = 25.6, 2 = 51.2, the next one adds on.
    data_buffer[22] = 2; // Network speed modifier in 0.1 increments
    data_buffer[23] = 255;

    data_buffer[0] = 28;
    data_buffer[1] = 2;
    data_buffer[5] = 16;

    let magic_number_2 = magic_checksum_processing(data_buffer, data_buffer_len);

    data_buffer[7] = (magic_number_2 >> 8) as u8;
    data_buffer[6] = magic_number_2 as u8;

    let mut final_buffer: [u8; 33] = [0u8; 33];
    final_buffer[0] = 0x0;
    final_buffer[1..33].copy_from_slice(&data_buffer);
    final_buffer
}

fn magic_temp_processing (temp : f32) -> i32 {
    let mut return_value: i32 = 0;
    if temp > 0.0 {
        return_value = (temp * 10.0).round() as i32 | 0;
    } else {
        return_value = (temp * -1.0 * 10.0).round() as i32 | 32768;
    }
    return_value
}

fn magic_checksum_processing (data_buffer : [u8; 32], data_buffer_len :usize ) -> i32 {
    let mut magic_number_2: i32 = 65535;
    for y in 0..data_buffer_len {
        magic_number_2 = magic_number_2 ^ ((data_buffer[y] as i32) << 8);

        for z in 0..8 {
            if (magic_number_2 & 32768) > 0 {
                magic_number_2 = (magic_number_2 << 1) ^ 4129;
            } else {
                magic_number_2 = magic_number_2 << 1;
            }
            magic_number_2 = magic_number_2 & 65535;
        }
    }
    magic_number_2
}
/* GETTER COMMANDS */

/// Construct a payload for getting the abi version of the keyboard
pub const fn get_version() -> [u8; 33] {
    let mut buf = [0u8; 33];
    buf[1] = 1;
    buf
}