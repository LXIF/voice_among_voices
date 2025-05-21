use hound::{WavReader, WavWriter};
use std::io::Cursor;

use crate::{
    AddVoiceNodeError, AudioParameters, AudioSampleMemory, SimulationParameters,
    VoiceNodeLocalMemory,
};

#[derive(Debug)]
pub struct SamplePosition {
    pub sample_id: u64,
    pub begins_at: f64, // normalized position of beginning of audio node versus tangent of angle => position - radius
    pub pan_position: f64, // signed normalized position as offset from center line at angle, will be the pan position
    pub sample_length_samples: u32, // length of node in samples, to calculate ends_at (we don't calculate here because normalized and we want sample accuracy)
}

/// Gets the sample length in ms
pub fn get_sample_length(audio_data: &Vec<u8>) -> Result<(u32, f64), AddVoiceNodeError> {
    let cursor = Cursor::new(audio_data);
    let reader = WavReader::new(cursor)
        .map_err(|e| AddVoiceNodeError::NotValidAudioFileError(e.to_string()))?;
    let sample_spec = reader.spec();
    let sample_length_samples = reader.duration();
    let sample_length_ms = sample_length_samples as f64 * 1000. / sample_spec.sample_rate as f64;

    Ok((sample_length_samples, sample_length_ms))
}

/// Generates the audio file per-angle
/// Imagine taking a tangent of the world-circle and dragging it across the world-circle
/// Whichever sample it hits gets played
pub fn generate_angle_file(
    angle: f64,
    nodes: &VoiceNodeLocalMemory,
    samples: &AudioSampleMemory,
    audio_params: &AudioParameters,
    sim_params: &SimulationParameters,
) -> Result<Vec<u8>, hound::Error> {
    // generate sample positions
    let sample_positions = generate_normalized_sample_positions(nodes, sim_params, angle);
    // generate stereo sample vectors
    let (left_samples, right_samples) =
        generate_audio_vectors(&sample_positions, audio_params, samples);

    // generate resulting file

    let angle_file = write_stereo_wav_to_vec(&audio_params, &left_samples, &right_samples);

    angle_file
}

fn generate_normalized_sample_positions(
    nodes: &VoiceNodeLocalMemory,
    sim_params: &SimulationParameters,
    angle: f64,
) -> Vec<SamplePosition> {
    let mut sample_positions: Vec<SamplePosition> = Vec::with_capacity(
        nodes
            .iter()
            .filter(|node| node.sample_id != u64::MAX)
            .collect::<Vec<_>>()
            .len() as usize,
    );
    let radius = sim_params.logical_radius;

    // calculate tangency points
    let (x_c, y_c) = tangency_points(radius, angle);

    sample_positions.extend(nodes.iter().filter_map(|node| {
        if node.sample_id == u64::MAX {
            return None;
        }
        let position = distance_from_tangent(angle, node.x, node.y, x_c, y_c) / (2. * radius);
        Some(SamplePosition {
            begins_at: (position - (node.radius / (2. * radius))).max(0.).min(1.),
            pan_position: signed_distance_from_center_line(angle, node.x, node.y, x_c, y_c)
                / radius,
            sample_id: node.sample_id,
            sample_length_samples: node.sample_length_samples,
        })
    }));

    sample_positions
}

fn generate_audio_vectors(
    sample_positions: &Vec<SamplePosition>,
    audio_params: &AudioParameters,
    samples: &AudioSampleMemory,
) -> (Vec<i16>, Vec<i16>) {
    // Convert fade duration from milliseconds to samples
    let fade_samples = (audio_params.fade_ms * audio_params.sample_rate / 1000) as usize;

    // create vectors with length, fill them with neutral => half of i16::MAX
    let total_length_samples: u64 =
        audio_params.total_length_ms as u64 * audio_params.sample_rate as u64 / 1000;

    let mut left_channel = vec![0i16; total_length_samples as usize];
    let mut right_channel = vec![0i16; total_length_samples as usize];

    // for every sample, go to the 'begins with' in each vector
    for sample_pos in sample_positions.iter() {
        // find the exact sample where it begins
        let start_sample = (sample_pos.begins_at * total_length_samples as f64)
            .round()
            .max(0.) as usize;

        let end_sample = (start_sample + sample_pos.sample_length_samples as usize - 1)
            .min(total_length_samples as usize - 1);
        // use reader to read sample
        let input_samples = read_wav(&samples.get(sample_pos.sample_id as u64).unwrap().sample);

        // figure out the panning multipliers
        let pan = sample_pos.pan_position;
        let left_gain = ((1.0 - pan) * std::f64::consts::FRAC_PI_4).cos();
        let right_gain = ((1.0 + pan) * std::f64::consts::FRAC_PI_4).cos();

        // loop over samples zipped with the slice of our left and right channels we want
        for (index, sample) in input_samples.iter().enumerate() {
            let sample_index = start_sample + index;
            if sample_index > end_sample {
                break;
            }

            // Determine fading
            let fade_factor = if index < fade_samples {
                index as f64 / fade_samples as f64
            } else if index >= input_samples.len() - fade_samples {
                (input_samples.len() - 1 - index) as f64 / fade_samples as f64
            } else {
                1.
            };

            // add the scaled sample to both sides
            let left_sample = (*sample as f64 * left_gain * fade_factor) as i16;
            let right_sample = (*sample as f64 * right_gain * fade_factor) as i16;

            left_channel[sample_index] = left_channel[sample_index]
                .saturating_add(left_sample)
                .clamp(i16::MIN, i16::MAX); //TODO: maybe add compression
            right_channel[sample_index] = right_channel[sample_index]
                .saturating_add(right_sample)
                .clamp(i16::MIN, i16::MAX);
        }
    }

    // loop through the positions where samples are to be inserted
    // add the sample to the existing sample at that point

    (left_channel, right_channel)
}

fn read_wav(audio_data: &Vec<u8>) -> Vec<i16> {
    let cursor = Cursor::new(audio_data);
    let mut reader = WavReader::new(cursor).unwrap();

    let result = reader
        .samples::<i16>()
        .map(|sample| sample.unwrap()) // TODO: perhaps improve error handling
        .collect();

    result
}

fn write_stereo_wav_to_vec(
    audio_params: &AudioParameters,
    left_samples: &Vec<i16>,
    right_samples: &Vec<i16>,
) -> Result<Vec<u8>, hound::Error> {
    assert_eq!(left_samples.len(), right_samples.len());

    let sample_rate = audio_params.sample_rate;

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut buffer, spec)?;
    let mut sample_writer = writer.get_i16_writer((left_samples.len() * 2) as u32);

    for (&left_sample, &right_sample) in left_samples.iter().zip(right_samples.iter()) {
        unsafe {
            sample_writer.write_sample_unchecked(left_sample);
            sample_writer.write_sample_unchecked(right_sample);
        }
    }

    sample_writer.flush()?;
    writer.finalize()?;

    let wav_data = buffer.into_inner();

    Ok(wav_data)
}

fn tangency_points(radius: f64, angle: f64) -> (f64, f64) {
    // calculate tangency points
    (
        radius * angle.to_radians().sin(),
        radius * angle.to_radians().cos(),
    )
}

fn distance_from_tangent(angle: f64, x: f64, y: f64, x_c: f64, y_c: f64) -> f64 {
    // find point of tangency
    // xc,yc = rc*cos(angle),rc*sin(angle)
    // slope of radius is y/x
    // slope of tangent (mtangent) is then -x/y
    // tangent equation is:
    // y-yc = mtangent*(x-xc)

    // Handle special cases where the tangent line is horizontal or vertical
    if angle % 180.0 == 0.0 {
        // Horizontal tangent (angle = 0 or 180)
        return (y - y_c).abs();
    } else if angle % 90.0 == 0.0 {
        // Vertical tangent (angle = 90 or 270)
        return (x - x_c).abs();
    }

    // tangent slope
    let m_tangent = -x_c / y_c;

    // Tangent line equation: y - y_c = m_tangent * (x - x_c)
    // Rewriting it as Ax + By + C = 0
    let a_tangent = m_tangent;
    let b_tangent = -1.;
    let c_tangent = -m_tangent * x_c + y_c;

    let distance_to_tangent = (a_tangent * x + b_tangent * y + c_tangent).abs()
        / (a_tangent * a_tangent + b_tangent * b_tangent).sqrt();

    distance_to_tangent
}

fn signed_distance_from_center_line(angle: f64, x: f64, y: f64, x_c: f64, y_c: f64) -> f64 {
    // Handle special cases where the tangent line is horizontal or vertical
    match angle {
        0. => {
            return -x;
        }
        90. => {
            return y;
        }
        180. => {
            return x;
        }
        270. => {
            return -y;
        }
        _ => {}
    }

    // Midpoint of the circle (assumed to be at (0, 0) for simplicity)
    let (x_m, y_m) = (0.0, 0.0);

    // Vector from point of tangency to the midpoint of the circle
    let v1 = (x_m - x_c, y_m - y_c);

    // Vector from point of tangency to the point (x, y)
    let v2 = (x - x_c, y - y_c);

    // 2D Cross product to determine the sign (v1.x * v2.y - v1.y * v2.x)
    let cross_product = v1.0 * v2.1 - v1.1 * v2.0;

    // Slope of the radius
    let m_center = if x_c.abs() < 1e-10 { 0.0 } else { y_c / x_c };

    // Distance from the center line (Ax + By + C = 0 form)
    let a_center = m_center;
    let b_center = -1.0;
    let c_center = 0.0;

    // Signed distance calculation
    let distance_to_center_line = (a_center * x + b_center * y + c_center).abs()
        / (a_center * a_center + b_center * b_center).sqrt();

    // Use the cross product to determine if the distance is positive or negative
    if cross_product < 0.0 {
        distance_to_center_line // Left of the center line
    } else {
        -distance_to_center_line // Right of the center line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SIMULATION_PARAMETERS;
    use crate::test_functions::*;
    use crate::{VoiceNodeLocal, SAMPLES_MEMORY, VOICE_NODES_MEMORY};
    use core::f64;
    use hound::WavReader;

    #[test]
    fn test_sample_length() {
        let expected_duration_ms = 4200;
        let sample_rate = 44100;
        let expected_duration_samples = expected_duration_ms * sample_rate / 1000;

        let test_wav_data = generate_test_wav(expected_duration_ms, sample_rate);

        let (actual_duration_samples, actual_duration_ms) =
            get_sample_length(&test_wav_data).unwrap();

        assert_eq!(actual_duration_ms, expected_duration_ms as f64);
        assert_eq!(actual_duration_samples, expected_duration_samples);
    }

    #[test]
    fn test_write_stereo_wav_to_vec_basic() {
        let (left_samples, right_samples) = generate_test_samples();
        let audio_params = generate_audio_params(44100); // CD-quality sample rate

        let wav_data = write_stereo_wav_to_vec(&audio_params, &left_samples, &right_samples)
            .expect("Failed to write WAV");

        // Verify that the generated WAV can be read and matches the original data
        let cursor = Cursor::new(wav_data);
        let mut reader = WavReader::new(cursor).expect("Failed to read WAV file");
        let spec = reader.spec();

        // Check WAV specification
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 44100);
        assert_eq!(spec.bits_per_sample, 16);

        let mut samples = reader.samples::<i16>();

        // Verify the left and right channel samples were written correctly
        assert_eq!(samples.next().unwrap().unwrap(), i16::MAX); // Left channel
        assert_eq!(samples.next().unwrap().unwrap(), i16::MIN); // Right channel
        assert_eq!(samples.next().unwrap().unwrap(), i16::MIN); // Left channel
        assert_eq!(samples.next().unwrap().unwrap(), i16::MAX); // Right channel
        assert_eq!(samples.next().unwrap().unwrap(), 0); // Left channel
        assert_eq!(samples.next().unwrap().unwrap(), 0); // Right channel
        assert_eq!(samples.next().unwrap().unwrap(), i16::MAX / 2); // Left channel
        assert_eq!(samples.next().unwrap().unwrap(), i16::MAX / 2); // Right channel
    }

    #[test]
    #[should_panic]
    fn test_write_stereo_wav_to_vec_mismatched_lengths() {
        let left_samples = vec![i16::MAX, i16::MIN];
        let right_samples = vec![i16::MIN]; // Different length

        let audio_params = generate_audio_params(44100);

        // This should panic due to mismatched lengths
        write_stereo_wav_to_vec(&audio_params, &left_samples, &right_samples)
            .expect("This should fail due to mismatched sample lengths");
    }

    #[test]
    fn test_write_stereo_wav_to_vec_empty() {
        let left_samples: Vec<i16> = vec![];
        let right_samples: Vec<i16> = vec![];
        let audio_params = generate_audio_params(44100);

        let wav_data = write_stereo_wav_to_vec(&audio_params, &left_samples, &right_samples)
            .expect("Failed to write WAV");

        // WAV should be empty but valid
        let cursor = Cursor::new(wav_data);
        let mut reader = WavReader::new(cursor).expect("Failed to read WAV file");
        let samples: Vec<i16> = reader.samples().collect::<Result<_, _>>().unwrap();

        assert_eq!(samples.len(), 0); // Ensure there are no samples
    }

    #[test]
    fn test_write_stereo_wav_to_vec_low_sample_rate() {
        let (left_samples, right_samples) = generate_test_samples();
        let audio_params = generate_audio_params(8000); // Low sample rate

        let wav_data = write_stereo_wav_to_vec(&audio_params, &left_samples, &right_samples)
            .expect("Failed to write WAV");

        // Verify that the generated WAV can be read and matches the original data
        let cursor = Cursor::new(wav_data);
        let reader = WavReader::new(cursor).expect("Failed to read WAV file");
        let spec = reader.spec();

        // Check WAV specification for low sample rate
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 8000);
        assert_eq!(spec.bits_per_sample, 16);
    }

    #[test]
    fn test_generate_sample_positions_sanity() {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            for node in generate_test_nodes(vec![(-25., 0.), (0., -25.)]).iter() {
                nodes.push(node).unwrap();
            }

            let angle = 0.0;
            let sample_positions =
                generate_normalized_sample_positions(nodes, &SIMULATION_PARAMETERS, angle);

            // There should be 2 sample positions generated since there are 2 nodes
            assert_eq!(sample_positions.len(), 2);

            // Verify the positions
            let sample_position_1 = &sample_positions[0];
            println!("{:#?}", sample_position_1);
            assert_eq!(sample_position_1.sample_id, 0); // Sample ID should match
            assert!(sample_position_1.begins_at >= 0.0 && sample_position_1.begins_at <= 1.0); // Position should be normalized
            assert!(
                sample_position_1.pan_position >= -1.0 && sample_position_1.pan_position <= 1.0
            ); // Pan position should be within [-1, 1]

            let sample_position_2 = &sample_positions[1];
            assert_eq!(sample_position_2.sample_id, 1);
            assert!(sample_position_2.begins_at >= 0.0 && sample_position_2.begins_at <= 1.0);
            println!("{}", sample_position_2.pan_position);
            assert!(
                sample_position_2.pan_position >= -1.0 && sample_position_2.pan_position <= 1.0
            );
        });
    }

    #[test]
    fn test_generate_sample_positions_basic() {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            for node in generate_test_nodes(vec![(0., 50.), (50., 0.), (0., -25.)]) {
                nodes.push(&node).unwrap();
            }
            println!("{:#?}", nodes);

            let angle = 0.0;
            let sample_positions =
                generate_normalized_sample_positions(nodes, &SIMULATION_PARAMETERS, angle);

            // There should be 3 sample positions generated since there are 3 nodes
            assert_eq!(sample_positions.len(), 3);

            // Verify the actual positions
            let sample_position_1 = &sample_positions[0];

            let sample_position_2 = &sample_positions[1];

            let sample_position_3 = &sample_positions[2];

            // first one should be at 0.
            assert_eq!(sample_position_1.begins_at, 0.);
            // pan 1 should be at 0.
            assert_eq!(sample_position_1.pan_position, 0.);

            // second one should be at 0.5
            assert_eq!(sample_position_2.begins_at, 0.4);
            // pan 2 should be at -1.
            assert_eq!(sample_position_2.pan_position, -1.);

            // third one should be at 0.5
            assert_eq!(sample_position_3.begins_at, 0.65);
            // pan 3 should be at -1.
            assert_eq!(sample_position_3.pan_position, 0.);
        });
    }

    #[test]
    fn test_generate_sample_positions_center_case() {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            nodes
                .push(&VoiceNodeLocal {
                    id: 1,
                    x: 0.0, //center
                    y: 0.0, //center
                    sample_id: 0,
                    radius: 10.0,
                    sample_length_samples: 44100 * 12,
                })
                .unwrap();

            let angle = 45.0; // Diagonal angle
            let sample_positions =
                generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, angle);

            assert_eq!(sample_positions.len(), 1);
            let sample_position = &sample_positions[0];
            assert_eq!(sample_position.sample_id, 0);
            println!("{}", sample_position.begins_at);
            println!("{}", sample_position.pan_position);
            assert!(approximately_equal(sample_position.begins_at, 0.4, 1e-6));
            assert_eq!(sample_position.pan_position, 0.);
        });
    }

    #[test]
    fn test_generate_sample_positions_no_samples() {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            let angle = 0.0;
            let sample_positions =
                generate_normalized_sample_positions(nodes, &SIMULATION_PARAMETERS, angle);

            // No sample positions should be generated
            assert_eq!(sample_positions.len(), 0);
        });
    }

    #[test]
    fn test_generate_sample_positions_varying_angles() {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            for node in generate_test_nodes(vec![(25., 50.), (50., 25.)]).iter() {
                nodes.push(node).unwrap();
            }

            let angle_0 = 0.0;
            let angle_90 = 90.0;
            let angle_180 = 180.0;
            let angle_270 = 270.0;

            let positions_0 =
                generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, angle_0);
            let positions_90 =
                generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, angle_90);
            let positions_180 =
                generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, angle_180);
            let positions_270 =
                generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, angle_270);

            // There should always be 2 sample positions, but their properties (position, pan) should vary
            assert_eq!(positions_0.len(), 2);
            assert_eq!(positions_90.len(), 2);
            assert_eq!(positions_180.len(), 2);
            assert_eq!(positions_270.len(), 2);

            // Check that positions change based on the angle
            assert_ne!(positions_0[0].begins_at, positions_90[0].begins_at);
            assert_ne!(positions_90[0].begins_at, positions_180[0].begins_at);
            assert_ne!(positions_180[0].begins_at, positions_270[0].begins_at);
        });
    }

    #[test]
    fn test_generate_sample_positions_cross() {
        VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
            for node in
                generate_test_nodes(vec![(0., 50.), (50., 0.), (0., -50.), (-50., 0.), (0., 0.)])
                    .iter()
            {
                nodes.push(node).unwrap();
            }

            let angle = 0.;

            for node in nodes.iter() {
                println!("{:#?}", node);
            }

            let positions =
                generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, angle);

            for position in positions.iter() {
                println!("{:#?}", position);
            }

            assert_eq!(positions[0].begins_at, 0.);
            assert_eq!(positions[1].begins_at, 0.4);
            assert_eq!(positions[2].begins_at, 0.9);
            assert_eq!(positions[3].begins_at, 0.4);
            assert_eq!(positions[4].begins_at, 0.4);
        });
    }

    #[test]
    fn test_generate_audio_vectors_basic() {
        SAMPLES_MEMORY.with_borrow_mut(|samples_memory| {
            let audio_params = AudioParameters {
                fade_ms: 0,            // Fade duration in milliseconds
                sample_rate: 44100,    // 44.1kHz
                total_length_ms: 1000, // 1 second total length
                max_sample_length_ms: 60000,
                chunk_size: 1024 * 1024,
            };
            let sample = generate_static_test_sample(1000.0, 44100, 0);
            let sample_positions = generate_test_sample_positions(&sample);
            samples_memory.push(&sample).unwrap();

            let (left_channel, right_channel) =
                generate_audio_vectors(&sample_positions, &audio_params, samples_memory);

            // Verify that the output vectors are of the correct length
            let expected_length = audio_params.total_length_ms * audio_params.sample_rate / 1000;
            assert_eq!(left_channel.len(), expected_length as usize);
            assert_eq!(right_channel.len(), expected_length as usize);

            // Since the pan position is 0 (center), both left and right channels should be equal
            for (left, right) in left_channel.iter().zip(right_channel.iter()) {
                assert_eq!(left, right);
            }
        });
    }

    #[test]
    fn test_generate_audio_vectors_positions() {
        let audio_params = AudioParameters {
            fade_ms: 0,           // Fade duration in milliseconds
            sample_rate: 44100,   // 44.1kHz
            total_length_ms: 100, // 0.1 second total length
            max_sample_length_ms: 10,
            chunk_size: 1024 * 1024,
        };

        SAMPLES_MEMORY.with_borrow_mut(|samples_memory| {
            VOICE_NODES_MEMORY.with_borrow_mut(|nodes| {
                for node in generate_short_test_nodes(vec![(0., 50.), (0., 0.), (0., -50.)]).iter()
                {
                    nodes.push(node).unwrap();
                }
                for i in 0..3 {
                    samples_memory
                        .push(&generate_static_test_sample(10., 441, i))
                        .unwrap();
                }

                let sample_positions =
                    generate_normalized_sample_positions(&nodes, &SIMULATION_PARAMETERS, 0.);

                let (left_channel, right_channel) =
                    generate_audio_vectors(&sample_positions, &audio_params, &samples_memory);

                // first and last and midpoint should be 11584
                // 1/3 and 2/3 should be 0
                let total_length_samples =
                    audio_params.total_length_ms * audio_params.sample_rate / 1000;

                let first_first_index = 0;
                let first_last_index = (sample_positions[0].sample_length_samples - 1) as usize;

                let mid_first_index =
                    (sample_positions[1].begins_at * total_length_samples as f64) as usize;
                let mid_last_index =
                    mid_first_index + sample_positions[1].sample_length_samples as usize - 1;

                let last_first_index =
                    (sample_positions[2].begins_at * total_length_samples as f64) as usize;
                let last_last_index = (total_length_samples - 1) as usize;

                let first_zero_point = (1. / 3. * total_length_samples as f64) as usize;
                let second_zero_point = (2. / 3. * total_length_samples as f64) as usize;

                assert_eq!(left_channel[first_first_index], 11584);
                assert_eq!(right_channel[first_first_index], 11584);

                assert_eq!(left_channel[first_last_index], 11584);
                assert_eq!(right_channel[first_last_index], 11584);

                assert_eq!(left_channel[first_last_index + 1], 0);
                assert_eq!(right_channel[first_last_index + 1], 0);

                assert_eq!(left_channel[mid_first_index - 1], 0);
                assert_eq!(right_channel[mid_first_index - 1], 0);

                assert_eq!(left_channel[mid_first_index], 11584);
                assert_eq!(right_channel[mid_first_index], 11584);

                assert_eq!(left_channel[mid_last_index], 11584);
                assert_eq!(right_channel[mid_last_index], 11584);

                assert_eq!(left_channel[mid_last_index + 1], 0);
                assert_eq!(right_channel[mid_last_index + 1], 0);

                assert_eq!(left_channel[last_first_index - 1], 0);
                assert_eq!(right_channel[last_first_index - 1], 0);

                assert_eq!(left_channel[last_first_index], 11584);
                assert_eq!(right_channel[last_first_index], 11584);

                assert_eq!(left_channel[last_last_index], 11584);
                assert_eq!(right_channel[last_last_index], 11584);

                assert_eq!(left_channel[first_zero_point], 0);
                assert_eq!(right_channel[first_zero_point], 0);

                assert_eq!(left_channel[second_zero_point], 0);
                assert_eq!(right_channel[second_zero_point], 0);
            });
        });
    }

    #[test]
    fn test_generate_audio_vectors_fading() {
        SAMPLES_MEMORY.with(|samples_memory| {
            let mut map = samples_memory.borrow_mut();
            let audio_params = AudioParameters {
                fade_ms: 100,          // Fade duration in milliseconds
                sample_rate: 44100,    // 44.1kHz
                total_length_ms: 1000, // 1 second total length
                max_sample_length_ms: 1000,
                chunk_size: 1024 * 1024,
            };
            map.push(&generate_static_test_sample(1000., 44100, 1))
                .unwrap();

            let sample_positions = vec![SamplePosition {
                // midpoint is in the middle
                begins_at: 0.0,    // Also begin at the very start
                pan_position: 0.0, // Center panning
                sample_id: 0,
                sample_length_samples: 44100,
            }];

            let fade_samples = (audio_params.fade_ms * audio_params.sample_rate / 1000) as usize;
            let start_fade =
                (sample_positions[0].begins_at * audio_params.total_length_ms as f64 * 44100.
                    / 1000.)
                    .round() as usize;

            let (left_channel, right_channel) =
                generate_audio_vectors(&sample_positions, &audio_params, &mut map);

            // for i in left_channel.iter() {
            //     print!("{i}");
            // }

            // Check that fading occurs in the beginning and end
            // The first fade_samples should ramp up
            for i in start_fade..start_fade + fade_samples {
                // println!("{} {}", left_channel[i], right_channel[i + 1]);
                assert!(left_channel[i] < left_channel[i + 1]);
                assert!(right_channel[i] < right_channel[i + 1]);
            }

            // The last fade_samples should ramp down
            let total_samples = left_channel.len();
            for i in (total_samples - fade_samples)..(total_samples - 1) {
                // println!("{} {}", left_channel[i], right_channel[i]);
                assert!(left_channel[i] > left_channel[i + 1]);
                assert!(right_channel[i] > right_channel[i + 1]);
            }
        });
    }

    #[test]
    fn test_generate_audio_vectors_no_sample() {
        SAMPLES_MEMORY.with_borrow_mut(|samples| {
            let audio_params = AudioParameters {
                fade_ms: 100,          // Fade duration in milliseconds
                sample_rate: 44100,    // 44.1kHz
                total_length_ms: 1000, // 1 second total length
                max_sample_length_ms: 60000,
                chunk_size: 1024 * 1024,
            };
            let sample_positions: Vec<SamplePosition> = vec![]; // No samples

            let (left_channel, right_channel) =
                generate_audio_vectors(&sample_positions, &audio_params, samples);

            // Both channels should be silent (all zeros)
            assert!(left_channel.iter().all(|&sample| sample == 0));
            assert!(right_channel.iter().all(|&sample| sample == 0));
        });
    }

    #[test]
    fn test_generate_audio_vectors_clipping_prevention() {
        SAMPLES_MEMORY.with_borrow_mut(|samples| {
            let audio_params = AudioParameters {
                fade_ms: 100,          // Fade duration in milliseconds
                sample_rate: 44100,    // 44.1kHz
                total_length_ms: 1000, // 1 second total length
                max_sample_length_ms: 60000,
                chunk_size: 1024 * 1024,
            };
            // map.insert(0 as u128, generate_static_test_sample(1000.0, 44100, 1));
            samples
                .push(&generate_extreme_test_sample(1000., 44100, 0))
                .unwrap();
            let sample_positions = generate_test_sample_positions(&samples.get(0 as u64).unwrap());

            let (left_channel, right_channel) =
                generate_audio_vectors(&sample_positions, &audio_params, samples);

            // Ensure that no values exceed the i16 range
            for &sample in left_channel.iter().chain(right_channel.iter()) {
                assert!(sample <= i16::MAX);
                assert!(sample >= i16::MIN);
            }
        });
    }

    #[test]
    fn test_generate_audio_vectors_pan_position() {
        SAMPLES_MEMORY.with_borrow_mut(|samples| {
            let audio_params = AudioParameters {
                fade_ms: 0,            // Fade duration in milliseconds
                sample_rate: 44100,    // 44.1kHz
                total_length_ms: 1000, // 1 second total length
                max_sample_length_ms: 60000,
                chunk_size: 1024 * 1024,
            };

            for i in 0..3 {
                samples
                    .push(&generate_static_test_sample(1000.0, 44100, i))
                    .unwrap();
            }

            // Test with left pan (-1.0), center (0.0), and right pan (1.0)

            let (left_channel, right_channel) = generate_audio_vectors(
                &vec![SamplePosition {
                    begins_at: 0.0,
                    sample_id: 0,
                    sample_length_samples: 44100,
                    pan_position: 1.0, // Full left pan
                }],
                &audio_params,
                samples,
            );

            // Left pan should result in higher values in the left channel
            for (left, right) in left_channel.iter().zip(right_channel.iter()) {
                println!("{left} {right}");
                assert!(left > right);
            }

            // Center pan should result in equal values for both channels
            let (center_left, center_right) = generate_audio_vectors(
                &vec![SamplePosition {
                    begins_at: 0.0,
                    sample_id: 0,
                    sample_length_samples: 44100,
                    pan_position: 0.0,
                }],
                &audio_params,
                samples,
            );
            for (left, right) in center_left.iter().zip(center_right.iter()) {
                assert_eq!(left, right);
            }

            // Right pan should result in higher values in the right channel
            let (right_left, right_right) = generate_audio_vectors(
                &vec![SamplePosition {
                    begins_at: 0.0,
                    sample_id: 0,
                    pan_position: -1.0,
                    sample_length_samples: 44100,
                }],
                &audio_params,
                samples,
            );
            for (left, right) in right_left.iter().zip(right_right.iter()) {
                assert!(right > left);
            }
        });
    }

    #[test]
    fn correct_distance_from_tangent() {
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (25., 0.);

            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert_eq!(d, 25.);
        }
        {
            let angle = 270.;
            let radius = 50.;
            let (x, y) = (25., 0.);
            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert_eq!(d, 75.);
        }
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (50., 0.);
            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert_eq!(d, 0.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (0., 25.);
            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert_eq!(d, 25.);
        }
        {
            let angle = 180.;
            let radius = 50.;
            let (x, y) = (0., 25.);
            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert_eq!(d, 75.);
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                45_f64.to_radians().sin() * 25.,
                45_f64.to_radians().cos() * 25.,
            );
            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert!(approximately_equal(d, 25., 1e-6));
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                45_f64.to_radians().sin() * -25.,
                45_f64.to_radians().cos() * -25.,
            );
            let (x_c, y_c) = tangency_points(radius, angle);

            let d = distance_from_tangent(angle, x, y, x_c, y_c);

            assert!(approximately_equal(d, 75., 1e-6));
        }
    }
    #[test]
    fn correct_signed_distance_from_center_line() {
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (0., 25.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, 25.);
        }
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (0., -25.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, -25.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (25., 0.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, -25.);
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                135_f64.to_radians().sin() * 25.,
                135_f64.to_radians().cos() * 25.,
            );

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert!(approximately_equal(d, -25., 1e-6));
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                135_f64.to_radians().sin() * -25.,
                135_f64.to_radians().cos() * -25.,
            );

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert!(approximately_equal(d, 25., 1e-6));
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (0., 25.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, 0.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (1., 25.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, -1.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (-1., 25.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, 1.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (-1., 25.);

            let (x_c, y_c) = tangency_points(radius, angle);
            let d = signed_distance_from_center_line(angle, x, y, x_c, y_c);

            assert_eq!(d, 1.);
        }
    }
}
