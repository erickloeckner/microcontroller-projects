// scale the u8 input value down to a range of (0..output_max)
pub fn u8_scale(input: u8, output_max: u8) -> u8 {
    ((input as u16 * (output_max as u16 + 1)) >> 8) as u8
}

// map the input value, which is in the range of (0..input_max),
// to a range of (0..output_max)
pub fn map_range(input: u8, input_max: u8, output_max: u8) -> u8 {
    let mut input_clip = input;
    if input > input_max {
        input_clip = input_max;
    }
    let percent_in = ((input_clip as u16 * 255) / (input_max as u16)) as u8;
    u8_scale(output_max, percent_in)
}
