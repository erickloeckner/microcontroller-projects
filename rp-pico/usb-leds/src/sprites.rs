use crate::prng::Prng;

#[derive(Copy, Clone)]
pub struct RandomSprite {
    value: f32,     // current value of sprite
    target: f32,    // next value that sprite changes to
    step_size: f32,
}

impl RandomSprite {
    pub fn new() -> Self {
        Self { value: 0.0, target: 0.0, step_size: 0.0 }
    }
    
    pub fn get_value(&self) -> f32 {
        self.value
    }
}

pub struct RandomSprites {
    value_scale: f32,             // range of sprite values, from 0.0..value_scale
    step_scale: f32,              // range of step size values, from (0..step_scale)
    len: usize,
    sprites: [RandomSprite; 1024],   // array of RandomSprite objects
}

impl RandomSprites {
    pub fn new(len: usize, value_scale: f32, step_scale: f32, rng: &mut Prng) -> Self {
        // Sprite count is limited to 1024 to match LEDs
        assert!(len <= 1024);
        let mut sprites = [RandomSprite::new(); 1024];

        for i in sprites.iter_mut() {
            i.value = (rng.rand_f32() * value_scale).max(0.0).min(1.0);
            i.target = (rng.rand_f32() * value_scale).max(0.0).min(1.0);
            i.step_size = (rng.rand_f32() * step_scale).max(0.001).min(1.0);
        }
        
        RandomSprites {
            value_scale: value_scale,
            step_scale: step_scale,
            len: len,
            sprites: sprites,
        }
    }
    
    pub fn run(&mut self, rng: &mut Prng) {
        for i in self.sprites.iter_mut() {
            if i.value > i.target {
                i.value = (i.value - i.step_size).max(i.target);
            }
            if i.value < i.target {
                i.value = (i.value + i.step_size).min(i.target);
            }
            if i.value == i.target {
                i.target = (rng.rand_f32() * self.value_scale).max(0.0).min(1.0);
                i.step_size = (rng.rand_f32() * self.step_scale).max(0.001).min(1.0);
            }
        }
    }

    pub fn get_sprites(&self) -> &[RandomSprite] {
        &self.sprites[0..self.len]
    }
}
