use embedded_hal::serial::Write;

fn filter_channel(channel: u8) -> u8 {
    (channel.max(1).min(16)) - 1
}

pub fn note_on<T: Write<u8>>(serial: &mut T, channel: u8, note: u8, velocity: u8) {
    let note_clip = note.min(127);
    let velocity_clip = velocity.min(127);
    let _ = serial.write(0x90 + filter_channel(channel));
    let _ = serial.write(note_clip);
    let _ = serial.write(velocity_clip);
    let _ = serial.flush();
}

pub fn note_off<T: Write<u8>>(serial: &mut T, channel: u8, note: u8, velocity: u8) {
    let note_clip = note.min(127);
    let velocity_clip = velocity.min(127);
    let _ = serial.write(0x80 + filter_channel(channel));
    let _ = serial.write(note_clip);
    let _ = serial.write(velocity_clip);
    let _ = serial.flush();
}

pub fn cc<T: Write<u8>>(serial: &mut T, channel: u8, number: u8, value: u8) {
    let number_clip = number.min(127);
    let value_clip = value.min(127);
    let _ = serial.write(0xB0 + filter_channel(channel));
    let _ = serial.write(number_clip);
    let _ = serial.write(value_clip);
    let _ = serial.flush();
}

pub fn aftertouch<T: Write<u8>>(serial: &mut T, channel: u8, value: u8) {
    let value_clip = value.min(127);
    let _ = serial.write(0xD0 + filter_channel(channel));
    let _ = serial.write(value_clip);
    let _ = serial.flush();
}

pub fn pitch_bend<T: Write<u8>>(serial: &mut T, channel: u8, value: u16) {
    let value_clip = value.min(16383);
    let lsb = (value_clip & 0x7F) as u8;
    let msb = (value_clip >> 7) as u8;
    let _ = serial.write(0xE0 + filter_channel(channel));
    let _ = serial.write(lsb);
    let _ = serial.write(msb);
    let _ = serial.flush();
}
