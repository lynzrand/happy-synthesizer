use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample,
};
use happy_synth::AdsrEnvelope;

const BPM: f32 = 194.0;

fn main() {
    let a4 = 440.0;
    let half_step = 2.0_f32.powf(1.0 / 12.0);
    let whole_step = half_step * half_step;

    // Notes
    let b4 = a4 * whole_step;
    let csharp4 = b4 * whole_step;
    let dsharp4 = csharp4 * whole_step;
    let e4 = dsharp4 * half_step;

    // Music score
    // (note frequency, duration in beats)
    let beat_time = 60.0 / BPM;
    let score = [
        (dsharp4, 2.0),
        (csharp4, 1.0),
        (b4, 2.0),
        (csharp4, 1.0),
        (dsharp4, 1.5),
        (e4, 0.5),
        (dsharp4, 1.0),
        (csharp4, 3.0),
        (dsharp4, 2.0),
        (csharp4, 1.0),
        (b4, 2.0),
        (csharp4, 1.0),
        (dsharp4, 1.5),
        (e4, 0.5),
        (dsharp4, 1.0),
        (csharp4, 3.0),
    ];
    // Convert to prefix sum
    let score = score
        .iter()
        .map(|x| Some(*x))
        .chain([None])
        .scan(0.0, |acc, note| match note {
            Some((note, duration)) => {
                let start = *acc;
                *acc += duration;
                Some((Some(note), start))
            }
            None => Some((None, *acc)),
        })
        .map(|(note, start)| (note, start * beat_time))
        .collect::<Vec<_>>();

    let duration = score.last().unwrap().1;
    eprintln!("Score duration: {}", duration);

    // Output
    let host = cpal::default_host();
    let out_dev = host
        .default_output_device()
        .expect("no output device available");
    let mut support_config_range = out_dev
        .supported_output_configs()
        .expect("No supported config");
    let config = support_config_range
        .find(|c| c.sample_format() == cpal::SampleFormat::F32)
        .expect("no supported output configuration")
        .with_max_sample_rate();

    // Synth settings
    let sample_rate = config.sample_rate().0 as f32;
    println!("Sample rate: {}", sample_rate);
    let channel_count = config.channels() as usize;
    println!("Channels: {}", channel_count);

    let cfg = happy_synth::Config {
        sample_rate,
        ..Default::default()
    };
    let adsr = AdsrEnvelope {
        attack: 0.01,
        decay: 0.5,
        sustain: 0.6,
        release: 0.2,
    };
    let osc = happy_synth::osc::saw::SawOscillator;
    let mut synth = happy_synth::Synth::new(cfg, osc, adsr, 256);

    eprintln!("Synth created");

    let delta_t = 1.0 / sample_rate;

    assert_eq!(config.sample_format(), cpal::SampleFormat::F32);

    let stream = out_dev
        .build_output_stream(
            &config.config(),
            {
                let mut time = 0.0; // Where we are in the score
                let mut next_note = 0; // The next note index to play
                let mut curr_note_id = None; // The note id returned by the synth
                move |d: &mut [f32], _info| {
                    d.fill(Sample::EQUILIBRIUM);

                    let samples_per_1ms = (sample_rate / 1000.0) as usize;
                    let actual_sample_per_1ms = samples_per_1ms * channel_count;
                    // Chop d into 1ms chunks so that we can update the note state
                    for ch in d.chunks_mut(actual_sample_per_1ms) {
                        // Update note state
                        // Check if we need to switch to the next note
                        if next_note < score.len() && time >= score[next_note].1 {
                            if let Some(note) = score[next_note].0 {
                                if let Some(id) = curr_note_id {
                                    synth.end_note(id);
                                }
                                curr_note_id = Some(synth.start_note(note, 0.5));
                            } else if let Some(id) = curr_note_id {
                                synth.end_note(id);
                            }
                            next_note += 1;
                        }

                        // Use the first channel to render the sound
                        let first_ch = &mut ch[0..samples_per_1ms];
                        synth.bookkeeping();
                        synth.render(first_ch);

                        // Copy the first channel to the rest of the channels
                        // work in reverse order to avoid overwriting
                        for i in (0..samples_per_1ms).rev() {
                            let start_idx = i * channel_count;
                            for j in 0..channel_count {
                                ch[start_idx + j] = ch[i]
                            }
                        }

                        time += delta_t * samples_per_1ms as f32;
                    }
                }
            },
            |e| panic!("{}", e),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    // Wait for the stream to finish
    std::thread::sleep(std::time::Duration::from_secs_f32(duration));

    eprintln!("WHY DID YOU PLAY HARUHIKAGE?!?!?!?!?!")
}
