use super::{
    sine::{SineOscillator, SineOscillatorState},
    Oscillator,
};

#[derive(Debug, Clone, Default)]
struct TableEntry {
    osc: SineOscillator,
    amplitude: f32,
}

#[derive(Debug, Clone, Default)]
pub struct HarmonicOscillator {
    table: Vec<TableEntry>,
}

impl HarmonicOscillator {
    pub fn new(amps: &[f32]) -> Self {
        let mut table = Vec::with_capacity(amps.len());
        for amp in amps {
            table.push(TableEntry {
                osc: SineOscillator,
                amplitude: *amp,
            });
        }
        Self { table }
    }
}

#[derive(Debug, Clone)]
pub struct HarmonicOscillatorState {
    states: Vec<SineOscillatorState>,
}

impl Oscillator for HarmonicOscillator {
    type State = HarmonicOscillatorState;

    fn create_state(&self) -> Self::State {
        let mut state = Vec::with_capacity(self.table.len());
        for _ in &self.table {
            state.push(SineOscillatorState::default());
        }
        HarmonicOscillatorState { states: state }
    }

    fn fill_samples(
        &self,
        state: &mut Self::State,
        buffer: &mut [f32],
        delta_t: f32,
        freq: f32,
        amp: f32,
    ) {
        buffer.fill(0.0);
        for (ix, (entry, state)) in self.table.iter().zip(state.states.iter_mut()).enumerate() {
            let multiplier = ix + 1;
            let entry_freq = freq * multiplier as f32;
            let entry_amp = entry.amplitude * amp / multiplier as f32;
            entry
                .osc
                .fill_samples(state, buffer, delta_t, entry_freq, entry_amp);
        }
    }
}
