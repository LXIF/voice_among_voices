use hound::{WavReader, WavSpec, WavWriter};
use std::io::Cursor;

use crate::{
    AddVoiceNodeError, AudioParameters, AudioSample, AudioSampleStore, SimulationParameters,
    VoiceNodeLocalStore,
};

/// Gets the sample length in ms
pub fn get_sample_length(audio_data: &Vec<u8>) -> Result<f64, AddVoiceNodeError> {
    let cursor = Cursor::new(audio_data);
    let reader = WavReader::new(cursor)
        .map_err(|e| AddVoiceNodeError::NotValidAudioFileError(e.to_string()))?;
    let sample_spec = reader.spec();
    let sample_length_ms = reader.duration() as f64 * 1000. / sample_spec.sample_rate as f64;

    Ok(sample_length_ms)
}

struct SamplePosition<'a> {
    sample: &'a AudioSample,
    position: f64,
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
) -> Vec<u8> {
    let mut sample_positions: Vec<SamplePosition> = vec![];

    let radius = sim_params.logical_width / 2.;
    // loop through nodes
    for node in nodes.iter() {
        // store sample reference with position between 0. and 1.
        let position =
            distance_from_tangent(angle, radius, node.x, node.y) / sim_params.logical_width;
        if let Some(sample) = samples.iter().find(|sample| sample.id == node.sample_id) {
            sample_positions.push(SamplePosition { position, sample });
        }
    }
    // sort samples by position
    sample_positions.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    // then create the file with hound

    todo!()
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

        let test_wav_data = generate_test_wav(expected_duration_ms, sample_rate);

        let actual_duration_ms = get_sample_length(&test_wav_data).unwrap();

        assert_eq!(actual_duration_ms, expected_duration_ms as f64);
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
