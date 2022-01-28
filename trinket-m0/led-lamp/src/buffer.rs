pub struct RingBuffer8 {
    buffer: [u8; 8],
    pos: usize,
}

impl RingBuffer8 {
    pub fn new() -> RingBuffer8 {
        RingBuffer8 {
            buffer: [0; 8],
            pos: 0,
        }
    }
    
    pub fn push(&mut self, value: u8) {
        self.buffer[self.pos] = value;
        self.pos = (self.pos + 1) % self.buffer.len();
    }
    
    pub fn mean(&mut self) -> u8 {
        let sum: u16 = self.buffer
            .iter()
            .fold(0, |s, &i| s + (i as u16));
        (sum / (self.buffer.len() as u16)) as u8
        
    }
}
