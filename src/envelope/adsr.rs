use crate::note::NoteState;

use super::Envelope;

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

impl Envelope for AdsrEnvelope {
    fn sample(&self, state: NoteState) -> f32 {
        match state {
            NoteState::Holding(time) => {
                if time < 0.0 {
                    0.0
                } else if time < self.attack {
                    1.0 - time / self.attack
                } else if time < self.attack + self.decay {
                    let decay_time = time - self.attack;
                    1.0 + (self.sustain - 1.0) * (decay_time / self.decay)
                } else {
                    self.sustain
                }
            }
            NoteState::Released(time) => {
                if time < self.release {
                    1.0 - time / self.release * self.sustain
                } else {
                    0.0
                }
            }
        }
    }

    fn note_ended(&self, state: NoteState) -> bool {
        match state {
            NoteState::Holding(_) => false,
            NoteState::Released(time) => time >= self.release,
        }
    }
}

impl AdsrEnvelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
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

/// An ADSR envelope that uses an exponential function for the phase transitions.
pub struct ExponentialAdsrEnvelope {
    /// The ending x value of the exponential function.
    pub end_x: f32,
    /// The actual envelope configuration.
    pub props: AdsrEnvelope,
}

/// Sample the point `t` within the inverse exponential function segment, which spans the interval
/// `0..end_x`, y value is `start` at `t=0` and `end` at `t=end_x`. This interval is compressed
/// into `0..1` and the function is sampled at `t`.
fn sample_exp(y_start: f32, y_end: f32, t_end: f32, t: f32) -> f32 {
    // The function has form of `y = a * e^(-x) + c`
    let a = (y_start - y_end) / (1.0 - (-t_end).exp());
    let c = y_start - a;
    // Sample the function at `t`
    let t = t * t_end;
    a * (-t).exp() + c
}

impl Envelope for ExponentialAdsrEnvelope {
    fn sample(&self, state: NoteState) -> f32 {
        match state {
            NoteState::Holding(time) => {
                if time < 0.0 {
                    0.0
                } else if time < self.props.attack {
                    let t = time / self.props.attack;
                    sample_exp(0.0, 1.0, self.end_x, t)
                } else if time < self.props.attack + self.props.decay {
                    let t = (time - self.props.attack) / self.props.decay;
                    sample_exp(1.0, self.props.sustain, self.end_x, t)
                } else {
                    self.props.sustain
                }
            }
            NoteState::Released(time) => {
                if time < self.props.release {
                    let t = time / self.props.release;
                    sample_exp(self.props.sustain, 0.0, self.end_x, t)
                } else {
                    0.0
                }
            }
        }
    }

    fn note_ended(&self, state: NoteState) -> bool {
        self.props.note_ended(state)
    }
}

#[test]
fn test_sample_exp() {
    use std::fmt::Write;

    // print a graph of the function
    let width = 80;
    for i in 0..width {
        let x = i as f32 / width as f32;
        let y = sample_exp(0.0, 1.0, 2.0, x);
        let amp = (y * width as f32) as i32;
        let mut wave = String::new();
        for i in 0..width {
            if i == amp {
                wave.push('+');
            } else {
                wave.push(' ');
            }
        }

        write!(wave, " {:.2}", y).unwrap();
        println!("{}", wave);
    }

    assert_eq!(sample_exp(0.0, 1.0, 2.0, 0.0), 0.0);
    assert_eq!(sample_exp(0.0, 1.0, 2.0, 1.0), 1.0);
}
