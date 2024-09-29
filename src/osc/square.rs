use super::Oscillator;

pub struct SquareOscillator;

#[derive(Debug, Clone, Default)]
pub struct SquareOscillatorState {
    /// A phase between 0 and 1.
    phase: f32,
}

impl Oscillator for SquareOscillator {
    type State = SquareOscillatorState;

    fn create_state(&self) -> Self::State {
        SquareOscillatorState::default()
    }

    fn fill_samples(
        &self,
        state: &mut Self::State,
        buffer: &mut [f32],
        delta_t: f32,
        freq: f32,
        amp: f32,
    ) {
        let increment = freq * delta_t;
        for sample in buffer.iter_mut() {
            *sample += if state.phase < 0.5 { amp } else { -amp };
            state.phase += increment;
            state.phase %= 1.0;
        }
    }
}
