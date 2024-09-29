use super::Oscillator;

pub struct SawOscillator;

#[derive(Debug, Clone, Default)]
pub struct SawOscillatorState {
    /// A phase between 0 and 1.
    phase: f32,
}

impl Oscillator for SawOscillator {
    type State = SawOscillatorState;

    fn create_state(&self) -> Self::State {
        SawOscillatorState::default()
    }

    fn fill_samples(
        &self,
        state: &mut Self::State,
        buffer: &mut [f32],
        delta_t: f32,
        freq: f32,
        amp: f32,
    ) {
        let increment = delta_t * freq;
        for sample in buffer.iter_mut() {
            *sample += (2.0 * state.phase - 1.0) * amp;
            state.phase += increment;
            state.phase %= 1.0;
        }
    }
}
