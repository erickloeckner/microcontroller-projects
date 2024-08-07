pub struct Prng {
    seed: u64,
    a: u32,
    c: u32,
    modulus: u32,
}

impl Prng {
    pub fn new(seed: u64) -> Self {
        Prng {seed: seed, a: 1103515245, c: 12345, modulus: 2147483648}
    }
    
    pub fn rand(&mut self) -> u64 {
        self.seed = (self.seed.wrapping_mul(self.a as u64).wrapping_add(self.c as u64)) % (self.modulus as u64);
        self.seed
    }
    
    pub fn rand_f32(&mut self) -> f32 {
        ((self.rand().to_le_bytes()[0] as f32) / (u8::MAX as f32)).max(0.0).min(1.0)
    }
}
