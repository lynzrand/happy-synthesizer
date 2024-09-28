mod note;

use std::collections::VecDeque;

use note::Note;
use rand::Rng;
use slotmap::SlotMap;

pub struct Config {
    /// The sample rate of the audio stream, in Hz.
    pub sample_rate: f32,
    /// The number of samples per buffer.
    pub buffer_size: usize,
    /// The number of samples to be copied from the previous buffer to the next.
    pub leftover_sample_count: usize,
}

pub const DEFAULT_SAMPLE_RATE: f32 = 44_100.0;
pub const DEFAULT_LEFTOVER_SAMPLE_COUNT: usize = 16;
pub const DEFAULT_BUFFER_SIZE: usize =
    DEFAULT_SAMPLE_RATE as usize / 200 + DEFAULT_LEFTOVER_SAMPLE_COUNT; // 5ms

impl Default for Config {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            leftover_sample_count: DEFAULT_LEFTOVER_SAMPLE_COUNT,
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

pub trait Oscillator {
    /// Fill the buffer with samples of the oscillator.
    ///
    /// The oscillator implementation should **add** its samples to the buffer, instead of
    /// overwriting them, in order to allow oscillators to be composable.
    ///
    /// - `delta_t` is the time between samples, in seconds.
    /// - `freq` is the base frequency of the oscillator, in Hz.
    /// - `amp` is the amplitude of the oscillator.
    fn fill_samples(&mut self, buffer: &mut [f32], delta_t: f32, freq: f32, amp: f32);
}

#[derive(Debug, Clone, Default)]
pub struct SineOscillator {
    /// A phase between 0 and 2pi.
    phase: f32,
}

impl Oscillator for SineOscillator {
    fn fill_samples(&mut self, buffer: &mut [f32], delta_t: f32, freq: f32, amp: f32) {
        let increment = 2.0 * std::f32::consts::PI * freq * delta_t;
        for sample in buffer.iter_mut() {
            *sample += self.phase.sin() * amp;
            self.phase += increment;
            self.phase %= 2.0 * std::f32::consts::PI;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SawOscillator {
    /// A phase between 0 and 1.
    phase: f32,
}

impl Oscillator for SawOscillator {
    fn fill_samples(&mut self, buffer: &mut [f32], delta_t: f32, freq: f32, amp: f32) {
        let increment = delta_t * freq;
        dbg!(increment, freq, delta_t);
        for sample in buffer.iter_mut() {
            *sample += (2.0 * self.phase - 1.0) * amp;
            self.phase += increment;
            self.phase %= 1.0;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SquareOscillator {
    /// A phase between 0 and 1.
    phase: f32,
}

impl Oscillator for SquareOscillator {
    fn fill_samples(&mut self, buffer: &mut [f32], delta_t: f32, freq: f32, amp: f32) {
        let increment = freq * delta_t;
        for sample in buffer.iter_mut() {
            *sample += if self.phase < 0.5 { amp } else { -amp };
            self.phase += increment;
            self.phase %= 1.0;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NoiseOscillator {
    rng: rand::rngs::ThreadRng,
}

impl Oscillator for NoiseOscillator {
    fn fill_samples(&mut self, buffer: &mut [f32], _delta_t: f32, _freq: f32, amp: f32) {
        for sample in buffer.iter_mut() {
            *sample += self.rng.gen_range(-1.0..1.0) * amp;
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TableEntry {
    oscillator: SineOscillator,
    amplitude: f32,
}

#[derive(Debug, Clone, Default)]
pub struct WaveTableOscillator {
    table: Vec<TableEntry>,
}

impl WaveTableOscillator {
    pub fn new(amps: &[f32]) -> Self {
        let mut table = Vec::with_capacity(amps.len());
        for amp in amps {
            table.push(TableEntry {
                oscillator: SineOscillator::default(),
                amplitude: *amp,
            });
        }
        Self { table }
    }
}

impl Oscillator for WaveTableOscillator {
    fn fill_samples(&mut self, buffer: &mut [f32], delta_t: f32, freq: f32, amp: f32) {
        buffer.fill(0.0);
        for (ix, entry) in self.table.iter_mut().enumerate() {
            let multiplier = ix + 1;
            let entry_freq = freq * multiplier as f32;
            let entry_amp = entry.amplitude * amp / multiplier as f32;
            entry
                .oscillator
                .fill_samples(buffer, delta_t, entry_freq, entry_amp);
        }
    }
}

/// An ADSR envelope configuration. All times are in seconds.
///
/// ```plaintext
/// amplitude
/// ^
/// |     /|\
/// |    / | \
/// |   /  |  +---------------+\ -  -  -  -  -  -  -  +
/// |  /   |  |               | \                     | Sustain
/// +-+----+--+---------------+--+------> time  -  -  +
///   |    |  |               +--+ Release
///   |    |  |               (note is released)
///   |    +--+ Decay
/// t=0----+ Attack
/// ```
pub struct AdsrEnvelope {
    /// The time it takes for the envelope to reach its maximum amplitude.
    pub attack: f32,
    /// The time it takes for the envelope to reach the sustain amplitude.
    pub decay: f32,
    /// The amplitude of the envelope while the note is held. 1 is the default amplitude.
    pub sustain: f32,
    /// The time it takes for the envelope to reach 0 after the note is released.
    pub release: f32,
}

pub enum NoteState {
    Holding(f32),
    Released(f32),
}

impl AdsrEnvelope {
    pub fn sample(&self, state: NoteState) -> f32 {
        match state {
            NoteState::Holding(time) => {
                if time < self.attack {
                    time / self.attack
                } else if time < self.attack + self.decay {
                    let decay_time = time - self.attack;
                    1.0 + (self.sustain - 1.0) * (decay_time / self.decay)
                } else {
                    self.sustain
                }
            }
            NoteState::Released(time) => {
                if time < self.release {
                    1.0 - time / self.release
                } else {
                    0.0
                }
            }
        }
    }

    pub fn immediate() -> Self {
        Self {
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
        }
    }
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self::immediate()
    }
}

pub struct Synth {
    /// The configuration of the synth.
    cfg: Config,

    /// The oscillator used to generate the sound.
    osc: Box<dyn Oscillator>,

    /// The ADSR envelope configuration.
    adsr: AdsrEnvelope,

    /// Notes currently being played.
    notes: note::NoteList,
}

impl Synth {
    pub fn new(
        cfg: Config,
        osc: Box<dyn Oscillator>,
        adsr: AdsrEnvelope,
        max_notes: usize,
    ) -> Self {
        Self {
            cfg,
            osc,
            adsr,
            notes: note::NoteList::new(max_notes),
        }
    }

    pub fn start_note(&mut self, freq: f32, amp: f32) -> note::NoteId {
        let note = Note {
            freq,
            amp,
            time: 0.0,
            held: true,
        };
        // note list helps maintain the capacity of notes
        self.notes.add(note)
    }

    pub fn end_note(&mut self, id: note::NoteId) {
        if let Some(note) = self.notes.get_mut(id) {
            note.held = false;
            note.time = 0.0;
        }
    }

    pub fn render(&mut self, buffer: &mut [f32]) {
        let delta_t = 1.0 / self.cfg.sample_rate;
        let total_time = buffer.len() as f32 * delta_t;
        let mut temp_buf = vec![0.0; buffer.len()];

        for note in self.notes.notes_mut() {
            self.osc
                .fill_samples(&mut temp_buf, delta_t, note.freq, note.amp);
            for (i, (out, sample)) in buffer.iter_mut().zip(temp_buf.iter()).enumerate() {
                let curr_time = i as f32 * delta_t;
                let amp = self.adsr.sample(if note.held {
                    NoteState::Holding(note.time + curr_time)
                } else {
                    NoteState::Released(note.time + curr_time)
                });
                *out += *sample * amp;
            }
            note.time += total_time;
        }
    }

    pub fn bookkeeping(&mut self) {
        self.notes.filter(|n| n.held || n.time < self.adsr.release);
    }
}
