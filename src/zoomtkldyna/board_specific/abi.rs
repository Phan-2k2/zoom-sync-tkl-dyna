// use crate::board_specific::types::{Icon, ScreenTheme, ZoomTklDynaError};
use crate::board_specific::types::{Icon};
use crate::screen::ScreenArgs;

#[allow(dead_code)]
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

// /// testing function, used for debugging
// pub fn generate_test_buffer() -> [u8; 33] {

//     let mut data_buffer = [0; 32];
//     let data_buffer_len = data_buffer.len();


//     let magic_number_2: i32 = magic_checksum_processing(data_buffer, data_buffer_len);

//     data_buffer[7] = (magic_number_2 >> 8) as u8;
//     data_buffer[6] = magic_number_2 as u8;

//     let mut final_buffer: [u8; 33] = [0u8; 33];
//     final_buffer[0] = 0x0;
//     final_buffer[1..33].copy_from_slice(&data_buffer);
//     final_buffer
// }

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

// A bunch of these values 
pub fn generate_sysinfo_buffer(cpu_temp: u8, gpu_temp: u32, speed_fan: u32, download: f32) -> [u8; 33] {
    let mut data_buffer = [0; 32];
    let data_buffer_len = data_buffer.len();

    let download_array = ((download * 10.0) as u32).to_le_bytes();
    let speed_fan_array = speed_fan.to_le_bytes();
    let gpu_temp_aray = gpu_temp.to_le_bytes();

    data_buffer[8] = 165;
    data_buffer[9] = 255;
    data_buffer[10] = 0;
    data_buffer[11] = 11;
    data_buffer[13] = 0; //i mean, if this is ever > 1 your cpu temp > 256c and well....that's not good
    data_buffer[14] = cpu_temp;
    data_buffer[15] = 0; //i mean, if this is ever > 1 your gpu temp > 256c and well....that's not good
    data_buffer[16] = gpu_temp_aray[0];
    data_buffer[17] = 0;
    data_buffer[18] = 0; //or 60? I think this is SSD temp or some other thermal identifier
    data_buffer[19] = speed_fan_array[1]; // This is fan rpm, 1 = 256, 2 = 512, the next one adds on.
    data_buffer[20] = speed_fan_array[0]; // Fan RPM modifier, +1 to above.
    data_buffer[21] = download_array[1]; // This is network speed, 1 = 25.6, 2 = 51.2, the next one adds on.
    data_buffer[22] = download_array[0]; // Network speed modifier in 0.1 increments
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

pub fn generate_screen_control_buffer (command: ScreenArgs) -> [u8; 33] {
    let command_value_1;
    let command_value_2;
    match command {
        ScreenArgs::Up => {
            command_value_1 = 2; 
            command_value_2 = 194;
        }
        ScreenArgs::Down => {
            command_value_1 = 1;
            command_value_2 = 195;
        }
        ScreenArgs::Return => {
            command_value_1 = 4;
            command_value_2 = 192;
        }
        ScreenArgs::Enter => {
            command_value_1 = 3;
            command_value_2 = 193;
        }
        // this resets the theme, removes all installed gifs and images for the screen
        ScreenArgs::Reset => {
            command_value_1 = 1;
            command_value_2 = 200;
        }
    }

    let mut data_buffer = [0; 32];
    let data_buffer_len = data_buffer.len();

    data_buffer[8] = 165;
    data_buffer[9] = 57;
    data_buffer[10] = 0;
    data_buffer[11] = 2;
    data_buffer[13] = command_value_1; // this is the command value
    data_buffer[14] = command_value_2; // second part of command value

    data_buffer[0] = 28;
    data_buffer[1] = 2;
    data_buffer[5] = 7;

    let magic_number_2 = magic_checksum_processing(data_buffer, data_buffer_len);

    data_buffer[7] = (magic_number_2 >> 8) as u8;
    data_buffer[6] = magic_number_2 as u8;

    let mut final_buffer: [u8; 33] = [0u8; 33];
    final_buffer[0] = 0x0;
    final_buffer[1..33].copy_from_slice(&data_buffer);
    final_buffer
}



fn magic_temp_processing (temp : f32) -> i32 {
    let return_value: i32;
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

        for _ in 0..8 {
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
