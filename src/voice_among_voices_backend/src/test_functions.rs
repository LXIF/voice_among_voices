use crate::{
    audio::SamplePosition, AudioParameters, AudioSample, VoiceNodeLocal, VoiceNodeLocalStore,
};
use hound::{WavSpec, WavWriter};
use std::io::Cursor;

pub fn generate_test_wav(duration_ms: u32, sample_rate: u32, normal_amplitude: f32) -> Vec<u8> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec).unwrap();

    let num_samples = (sample_rate * duration_ms / 1000) as usize;
    let amplitude = i16::MAX as f32 * normal_amplitude;

    for t in 0..num_samples {
        let sample = ((t as f32 / sample_rate as f32) * 440. * 2. * std::f32::consts::PI).sin();
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }

    writer.finalize().unwrap();

    buffer
}

// needed because floats and trigo aren't perfect lol
pub fn approximately_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

// Helper function to generate simple sample data
pub fn generate_test_samples() -> (Vec<i16>, Vec<i16>) {
    let left_samples = vec![i16::MAX, i16::MIN, 0, i16::MAX / 2];
    let right_samples = vec![i16::MIN, i16::MAX, 0, i16::MAX / 2];
    (left_samples, right_samples)
}

pub fn generate_audio_params(sample_rate: u32) -> AudioParameters {
    AudioParameters {
        sample_rate,
        max_sample_length_ms: 100,
        total_length_ms: 1000, // 1 second
        fade_ms: 0,            // No fade
        chunk_size: 1024 * 1024,
    }
}

pub fn generate_test_nodes(coordinates: Vec<(f64, f64)>) -> VoiceNodeLocalStore {
    let mut test_nodes: VoiceNodeLocalStore = vec![];

    for (id, (x, y)) in coordinates.iter().enumerate() {
        test_nodes.push(VoiceNodeLocal {
            id,
            x: *x,
            y: *y,
            sample_id: id as u64,
            radius: 10.,
            sample_length_samples: 44100 * 20,
        });
    }

    test_nodes
}

pub fn generate_short_test_nodes(coordinates: Vec<(f64, f64)>) -> VoiceNodeLocalStore {
    let mut test_nodes: VoiceNodeLocalStore = vec![];

    for (id, (x, y)) in coordinates.iter().enumerate() {
        test_nodes.push(VoiceNodeLocal {
            id,
            x: *x,
            y: *y,
            sample_id: id as u64,
            radius: 10.,
            sample_length_samples: 441,
        });
    }

    test_nodes
}

pub fn generate_static_test_sample(
    sample_length_ms: f64,
    sample_length_samples: u32,
    id: usize,
) -> AudioSample {
    // Define the WAV format (mono, 16-bit, 44.1kHz)
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Create a buffer to hold the WAV data
    let mut buffer = Vec::new();
    let mut writer = hound::WavWriter::new(Cursor::new(&mut buffer), spec).unwrap();

    // Generate a simple sine wave or arbitrary sample data

    let num_samples = (sample_length_ms / 1000.0 * spec.sample_rate as f64) as u32;
    for _ in 0..num_samples {
        let sample = i16::MAX / 2;
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();

    AudioSample {
        id: id as u64,
        sample: buffer, // Use the buffer that contains the valid WAV data
        sample_length_ms,
        sample_length_samples,
    }
}

pub fn generate_extreme_test_sample(
    sample_length_ms: f64,
    sample_length_samples: u32,
    id: usize,
) -> AudioSample {
    // Define the WAV format (mono, 16-bit, 44.1kHz)
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Create a buffer to hold the WAV data
    let mut buffer = Vec::new();
    let mut writer = hound::WavWriter::new(Cursor::new(&mut buffer), spec).unwrap();

    let num_samples = (sample_length_ms / 1000.0 * spec.sample_rate as f64) as u32;
    for _ in 0..num_samples {
        let sample = i16::MAX;
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();

    AudioSample {
        id: id as u64,
        sample: buffer, // Use the buffer that contains the valid WAV data
        sample_length_ms,
        sample_length_samples,
    }
}

pub fn generate_test_sample_positions(sample: &AudioSample) -> Vec<SamplePosition> {
    vec![SamplePosition {
        // Start at the beginning of the vector
        begins_at: 0.0,    // Also begin at the very start
        pan_position: 0.0, // Center panning
        sample_id: sample.id,
        sample_length_samples: sample.sample_length_samples,
    }]
}
