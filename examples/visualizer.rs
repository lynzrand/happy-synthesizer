use happy_synth::{envelope::adsr::AdsrEnvelope, osc::sine::SineOscillator, Config, Synth};

fn main() {
    let cfg = Config {
        sample_rate: 4000.0,
        ..Default::default()
    };
    let osc = SineOscillator;
    let adsr = AdsrEnvelope {
        attack: 0.01,
        decay: 0.01,
        sustain: 0.5,
        release: 0.1,
    };
    let mut synth = Synth::new(cfg, osc, adsr, 256);

    let mut out_buf = vec![0.0f32; 256];

    synth.start_note(440.0, 0.5);
    synth.render(&mut out_buf);

    for sample in out_buf.iter() {
        // construct a waveform
        let width = 80;
        let zero = width / 2;
        let amp = (sample * zero as f32) as i32;
        let mut wave = String::new();
        for i in 0..width {
            if i == zero {
                wave.push('|');
            } else if i == zero + amp {
                wave.push('+');
            } else {
                wave.push(' ');
            }
        }
        println!("{}", wave);
    }
}
