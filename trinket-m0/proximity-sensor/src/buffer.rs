pub struct RingBuffer<T, const N: usize> {
    buffer: [T; N],
    pos: usize,
}

impl<const N: usize> RingBuffer<u8, N> {
    pub fn new() -> Self {
        RingBuffer {
            buffer: [0; N],
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

impl<const N: usize> RingBuffer<u16, N> {
    pub fn new() -> Self {
        RingBuffer {
            buffer: [0; N],
            pos: 0,
        }
    }
    
    pub fn push(&mut self, value: u16) {
        self.buffer[self.pos] = value;
        self.pos = (self.pos + 1) % self.buffer.len();
    }
    
    pub fn mean(&mut self) -> u16 {
        let sum: u32 = self.buffer
            .iter()
            .fold(0, |s, &i| s + (i as u32));
        (sum / (self.buffer.len() as u32)) as u16
    }
}

impl<const N: usize> RingBuffer<u32, N> {
    pub fn new() -> Self {
        RingBuffer {
            buffer: [0; N],
            pos: 0,
        }
    }
    
    pub fn push(&mut self, value: u32) {
        self.buffer[self.pos] = value;
        self.pos = (self.pos + 1) % self.buffer.len();
    }
    
    pub fn mean(&mut self) -> u32 {
        let sum: u64 = self.buffer
            .iter()
            .fold(0, |s, &i| s + (i as u64));
        (sum / (self.buffer.len() as u64)) as u32
    }
}
