use hound::{WavReader, WavSpec, WavWriter};
use std::io::Cursor;

pub fn get_sample_length(audio_data: &Vec<u8>) -> hound::Result<f64> {
    let cursor = Cursor::new(audio_data);
    let reader = WavReader::new(cursor)?;
    let sample_spec = reader.spec();
    let sample_length_ms = reader.duration() as f64 * 1000. / sample_spec.sample_rate as f64;

    Ok(sample_length_ms)
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
    use super::*;

    #[test]
    fn test_sample_length() {
        let expected_duration_ms = 4200;
        let sample_rate = 44100;

        let test_wav_data = generate_test_wav(expected_duration_ms, sample_rate);

        let actual_duration_ms = get_sample_length(&test_wav_data).unwrap();

        assert_eq!(actual_duration_ms, expected_duration_ms as f64);
    }
}
