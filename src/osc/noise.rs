use super::Oscillator;

use rand::Rng;

pub struct NoiseOscillator;

#[derive(Debug, Clone, Default)]
pub struct NoiseOscillatorState {
    rng: rand::rngs::ThreadRng,
}

impl Oscillator for NoiseOscillator {
    type State = NoiseOscillatorState;

    fn create_state(&self) -> Self::State {
        NoiseOscillatorState {
            rng: rand::thread_rng(),
        }
    }

    fn fill_samples(
        &self,
        state: &mut NoiseOscillatorState,
        buffer: &mut [f32],
        _delta_t: f32,
        _freq: f32,
        amp: f32,
    ) {
        for sample in buffer.iter_mut() {
            *sample += state.rng.gen_range(-1.0..1.0) * amp;
        }
    }
}
