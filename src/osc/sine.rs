use super::Oscillator;

#[derive(Debug, Clone, Default)]
pub struct SineOscillator;

#[derive(Debug, Clone, Default)]
pub struct SineOscillatorState {
    /// A phase between 0 and 2pi.
    phase: f32,
}

impl Oscillator for SineOscillator {
    type State = SineOscillatorState;

    fn create_state(&self) -> Self::State {
        SineOscillatorState::default()
    }

    fn fill_samples(
        &self,
        state: &mut Self::State,
        buffer: &mut [f32],
        delta_t: f32,
        freq: f32,
        amp: f32,
    ) {
        let increment = 2.0 * std::f32::consts::PI * freq * delta_t;
        for sample in buffer.iter_mut() {
            *sample += state.phase.sin() * amp;
            state.phase += increment;
            state.phase %= 2.0 * std::f32::consts::PI;
        }
    }
}
