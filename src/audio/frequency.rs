use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

pub struct FrequencyDetector {
    detector: McLeodDetector<f64>,
    buffer: Vec<f64>,
    size: usize,
    power_threshold: f64,
    clarity_threshold: f64,
}

impl FrequencyDetector {
    pub fn new(size: usize, padding: usize, power_threshold: f64, clarity_threshold: f64) -> Self {
        Self {
            detector: McLeodDetector::new(size, padding),
            buffer: vec![0.0; size],
            size,
            power_threshold,
            clarity_threshold,
        }
    }

    pub fn detect(&mut self, samples: &[f32], sample_rate: u32) -> Option<(f32, f32)> {
        if samples.is_empty() {
            return None;
        }

        if samples.len() >= self.size {
            let start = samples.len() - self.size;
            for (dst, src) in self.buffer.iter_mut().zip(&samples[start..]) {
                *dst = *src as f64;
            }
        } else {
            let pad = self.size - samples.len();
            for dst in self.buffer.iter_mut().take(pad) {
                *dst = 0.0;
            }
            for (dst, src) in self.buffer.iter_mut().skip(pad).zip(samples.iter()) {
                *dst = *src as f64;
            }
        }

        let pitch = self.detector.get_pitch(
            &self.buffer,
            sample_rate as usize,
            self.power_threshold,
            self.clarity_threshold,
        )?;

        Some((pitch.frequency as f32, pitch.clarity as f32))
    }
}
