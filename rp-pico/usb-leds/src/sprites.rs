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

pub struct RandomSprites<const N: usize> {
    value_scale: f32,             // range of sprite values, from 0.0..value_scale
    step_scale: f32,              // range of step size values, from (0..step_scale)
    sprites: [RandomSprite; N],   // array of RandomSprite objects
}

impl<const N: usize> RandomSprites<N> {
    pub fn new(value_scale: f32, step_scale: f32, rng: &mut Prng) -> Self {
        let mut sprites = [RandomSprite::new(); N];

        for i in sprites.iter_mut() {
            i.value = (rng.rand_f32() * value_scale).max(0.0).min(1.0);
            i.target = (rng.rand_f32() * value_scale).max(0.0).min(1.0);
            i.step_size = (rng.rand_f32() * step_scale).max(0.001).min(1.0);
        }
        
        RandomSprites {
            value_scale: value_scale,
            step_scale: step_scale,
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
        &self.sprites
    }
}
