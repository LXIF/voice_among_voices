use hound::{WavReader, WavSpec, WavWriter};
use ic_cdk::api::time;
use std::io::Cursor;

use crate::{
    AddVoiceNodeError, AudioParameters, AudioSample, AudioSampleStore, SimulationParameters,
    VoiceNodeLocalStore,
};

struct SamplePosition<'a> {
    sample: &'a AudioSample,
    position: f64, // normalized position of center of audio node versus tangent of angle, the position it will have in the file
    begins_at: f64, // normalized position of beginning of audio node versus tangent of angle => position - radius
    pan_position: f64, // signed normalized position as offset from center line at angle, will be the pan position
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
    nodes: &VoiceNodeLocalStore,
    samples: &AudioSampleStore,
    audio_params: &AudioParameters,
    sim_params: &SimulationParameters,
) -> Result<Vec<u8>, hound::Error> {
    let mut sample_positions: Vec<SamplePosition> = vec![];

    let radius = sim_params.logical_width / 2.;
    // loop through nodes
    for node in nodes.iter() {
        // store sample reference with normalized position between 0. and 1. and normalized pan position between -1. and 1.
        let position =
            distance_from_tangent(angle, radius, node.x, node.y) / sim_params.logical_width;
        let begins_at = position - (node.radius / sim_params.logical_width);
        let pan_position = signed_distance_from_center_line(angle, radius, node.x, node.y) / radius;

        if let Some(sample) = samples.iter().find(|sample| sample.id == node.sample_id) {
            sample_positions.push(SamplePosition {
                position,
                begins_at,
                sample,
                pan_position,
            });
        }
    }

    // generate stereo sample vectors
    let (left_samples, right_samples) = generate_audio_vectors(&sample_positions, audio_params);

    // generate resulting file

    let angle_file = write_stereo_wav_to_vec(&audio_params, &left_samples, &right_samples);

    angle_file
}

fn generate_audio_vectors(
    sample_positions: &Vec<SamplePosition>,
    audio_params: &AudioParameters,
) -> (Vec<i16>, Vec<i16>) {
    // create vectors with length, fill them with neutral => half of i16::MAX
    let total_length_samples = audio_params.total_length_ms * audio_params.sample_rate / 1000;
    let mut left_channel = vec![0i16; total_length_samples as usize];
    let mut right_channel = vec![0i16; total_length_samples as usize];

    // for every sample, go to the 'begins with' in each vector
    for sample_pos in sample_positions.iter() {
        // find the exact sample where it begins
        let start_sample = (sample_pos.begins_at * total_length_samples as f64)
            .round()
            .max(0.) as usize;
        let end_sample = (start_sample + sample_pos.sample.sample_length_samples as usize - 1)
            .min(total_length_samples as usize - 1);
        // use reader to read sample
        let input_samples = read_wav(&sample_pos.sample.sample);
        // loop over samples zipped with the slice of our left and right channels we want
        for (sample, (left, right)) in input_samples //TODO: this could in principle be parallelized, would require keeping the vecs in an arc/mutex
            .iter()
            .zip(
                left_channel[start_sample..=end_sample] //TODO: perhaps add some check to make sure we're never out of bounds?
                    .iter_mut()
                    .zip(right_channel[start_sample..=end_sample].iter_mut()),
            )
        {
            // figure out the panning multipliers
            let pan = sample_pos.pan_position;
            let left_gain = (0.5 * (1.0 + pan) * std::f64::consts::PI).cos();
            let right_gain = (0.5 * (1.0 - pan) * std::f64::consts::PI).cos();
            // add the scaled sample to both sides
            let left_sample = (*sample as f64 * left_gain) as i16;
            let right_sample = (*sample as f64 * right_gain) as i16;

            *left = left.wrapping_add(left_sample).clamp(i16::MIN, i16::MAX);
            *right = right.wrapping_add(right_sample).clamp(i16::MIN, i16::MAX);
        }
    }

    // loop through the positions where samples are to be inserted
    // add the sample to the existing sample at that point

    todo!()
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

    for (&left_sample, &right_sample) in left_samples.iter().zip(right_samples.iter()) {
        writer.write_sample(left_sample)?;
        writer.write_sample(right_sample)?;
    }

    writer.finalize()?;

    let wav_data = buffer.into_inner();

    Ok(wav_data)
}

pub fn generate_test_wav(duration_ms: u32, sample_rate: u32) -> Vec<u8> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec).unwrap();

    let num_samples = (sample_rate * duration_ms / 1000) as usize;
    let amplitude = i16::MAX as f32;

    for t in 0..num_samples {
        let sample = ((t as f32 / sample_rate as f32) * 440. * 2. * std::f32::consts::PI).sin();
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }

    writer.finalize().unwrap();

    buffer
}

fn distance_from_tangent(angle: f64, radius: f64, x: f64, y: f64) -> f64 {
    // find point of tangency
    // xc,yc = rc*cos(angle),rc*sin(angle)
    // slope of radius is y/x
    // slope of tangent (mtangent) is then -x/y
    // tangent equation is:
    // y-yc = mtangent*(x-xc)

    // point of tangency
    let x_c = radius * angle.to_radians().sin();
    let y_c = radius * angle.to_radians().cos();

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

fn signed_distance_from_center_line(angle: f64, radius: f64, x: f64, y: f64) -> f64 {
    // Calculate point of tangency
    let x_c = radius * angle.to_radians().sin();
    let y_c = radius * angle.to_radians().cos();

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
    use core::f64;

    use super::*;

    // needed because floats and trigo aren't perfect lol
    fn approximately_equal(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

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
    fn correct_distance_from_tangent() {
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (25., 0.);

            let d = distance_from_tangent(angle, radius, x, y);

            assert_eq!(d, 25.);
        }
        {
            let angle = 270.;
            let radius = 50.;
            let (x, y) = (25., 0.);

            let d = distance_from_tangent(angle, radius, x, y);

            assert_eq!(d, 75.);
        }
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (50., 0.);

            let d = distance_from_tangent(angle, radius, x, y);

            assert_eq!(d, 0.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (0., 25.);

            let d = distance_from_tangent(angle, radius, x, y);

            assert_eq!(d, 25.);
        }
        {
            let angle = 180.;
            let radius = 50.;
            let (x, y) = (0., 25.);

            let d = distance_from_tangent(angle, radius, x, y);

            assert_eq!(d, 75.);
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                45_f64.to_radians().sin() * 25.,
                45_f64.to_radians().cos() * 25.,
            );

            let d = distance_from_tangent(angle, radius, x, y);

            assert!(approximately_equal(d, 25., 1e-6));
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                45_f64.to_radians().sin() * -25.,
                45_f64.to_radians().cos() * -25.,
            );

            let d = distance_from_tangent(angle, radius, x, y);

            assert!(approximately_equal(d, 75., 1e-6));
        }
    }
    #[test]
    fn correct_signed_distance_from_center_line() {
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (0., 25.);

            let d = signed_distance_from_center_line(angle, radius, x, y);

            assert_eq!(d, 25.);
        }
        {
            let angle = 90.;
            let radius = 50.;
            let (x, y) = (0., -25.);

            let d = signed_distance_from_center_line(angle, radius, x, y);

            assert_eq!(d, -25.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (25., 0.);

            let d = signed_distance_from_center_line(angle, radius, x, y);

            assert_eq!(d, -25.);
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                135_f64.to_radians().sin() * 25.,
                135_f64.to_radians().cos() * 25.,
            );

            let d = signed_distance_from_center_line(angle, radius, x, y);

            assert!(approximately_equal(d, -25., 1e-6));
        }
        {
            let angle = 45.;
            let radius = 50.;
            let (x, y) = (
                135_f64.to_radians().sin() * -25.,
                135_f64.to_radians().cos() * -25.,
            );

            let d = signed_distance_from_center_line(angle, radius, x, y);

            // assert!(approximately_equal(d, 75., 1e-6));
            assert_eq!(d, 25.);
        }
        {
            let angle = 0.;
            let radius = 50.;
            let (x, y) = (0., 25.);

            let d = signed_distance_from_center_line(angle, radius, x, y);

            assert_eq!(d, 0.);
        }
    }
}
