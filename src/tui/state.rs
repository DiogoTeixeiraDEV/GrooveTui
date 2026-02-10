

use anyhow::Result;
use crossbeam_channel::Receiver;

use crate::audio::capture::{AudioCapture, CaptureConfig, list_input_devices};
use crate::audio::frequency::FrequencyDetector;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TuningMode {
    Auto,
    Manual,
}

pub const TUNER_STRINGS: [(&str, f32); 6] = [
    ("E2", 82.0),
    ("A2", 110.0),
    ("D3", 146.8),
    ("G3", 196.0),
    ("B3", 246.9),
    ("E4", 329.63),
];


pub struct TunerState {
    
    available_devices: Vec<String>,
    
    selected_device_index: usize,
    
    capture: Option<AudioCapture>,
    
    sample_rx: Option<Receiver<Vec<f32>>>,
    
    current_level: f32,
    
    peak_level: f32,
    
    sample_rate: u32,
    
    is_capturing: bool,
    
    error_message: Option<String>,

    waveform_buffer: Vec<f32>,
    last_smooth: f32,
    has_last_smooth: bool,

    raw_buffer: Vec<f32>,
    last_freq_smooth: f32,
    has_last_freq_smooth: bool,

    frequency_detector: FrequencyDetector,
    current_frequency: Option<f32>,
    current_clarity: Option<f32>,

    input_gain: f32,

    tuning_mode: TuningMode,
    selected_string_index: usize,
}

impl TunerState {
    const WAVEFORM_CAPACITY: usize = 256;
    const DETECTOR_SIZE: usize = 2048;
    const POWER_THRESHOLD: f64 = 0.5;
    const CLARITY_THRESHOLD: f64 = 0.6;
    const FREQUENCY_SMOOTH_ALPHA: f32 = 0.25;
    const WAVEFORM_DECIMATION: usize = 4;
    const WAVEFORM_DISPLAY_GAIN: f32 = 3.0;
    const WAVEFORM_SMOOTH_ALPHA: f32 = 0.2;
    const INPUT_GAIN_MIN: f32 = 0.0;
    const INPUT_GAIN_MAX: f32 = 10.0;
    const INPUT_GAIN_STEP: f32 = 0.5;
    const INPUT_GAIN_DEFAULT: f32 = 8.0;
    
    pub fn new() -> Self {
        let available_devices = list_input_devices().unwrap_or_default();
        
        Self {
            available_devices,
            selected_device_index: 0,
            capture: None,
            sample_rx: None,
            current_level: 0.0,
            peak_level: 0.0,
            sample_rate: 44100,
            is_capturing: false,
            error_message: None,
            waveform_buffer: Vec::with_capacity(Self::WAVEFORM_CAPACITY),
            last_smooth: 0.0,
            has_last_smooth: false,
            raw_buffer: Vec::with_capacity(Self::raw_buffer_capacity()),
            last_freq_smooth: 0.0,
            has_last_freq_smooth: false,
            frequency_detector: FrequencyDetector::new(
                Self::DETECTOR_SIZE,
                Self::detector_padding(),
                Self::POWER_THRESHOLD,
                Self::CLARITY_THRESHOLD,
            ),
            current_frequency: None,
            current_clarity: None,
            input_gain: Self::INPUT_GAIN_DEFAULT,
            tuning_mode: TuningMode::Auto,
            selected_string_index: 0,
        }
    }

    
    pub fn selected_device(&self) -> Option<&str> {
        self.available_devices.get(self.selected_device_index).map(|s| s.as_str())
    }

    pub fn current_level(&self) -> f32 {
        self.current_level
    }

    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }

    pub fn is_capturing(&self) -> bool {
        self.is_capturing
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn current_frequency(&self) -> Option<f32> {
        self.current_frequency
    }

    pub fn input_gain(&self) -> f32 {
        self.input_gain
    }

    pub fn tuning_mode(&self) -> TuningMode {
        self.tuning_mode
    }

    pub fn toggle_tuning_mode(&mut self) {
        self.tuning_mode = match self.tuning_mode {
            TuningMode::Auto => TuningMode::Manual,
            TuningMode::Manual => TuningMode::Auto,
        };
    }

    pub fn selected_string(&self) -> (&'static str, f32) {
        TUNER_STRINGS[self.selected_string_index]
    }

    pub fn selected_string_label(&self) -> &'static str {
        self.selected_string().0
    }

    pub fn selected_string_target(&self) -> f32 {
        self.selected_string().1
    }

    pub fn next_string(&mut self) {
        self.selected_string_index =
            (self.selected_string_index + 1) % TUNER_STRINGS.len();
    }

    pub fn prev_string(&mut self) {
        if self.selected_string_index == 0 {
            self.selected_string_index = TUNER_STRINGS.len() - 1;
        } else {
            self.selected_string_index -= 1;
        }
    }

    
    pub fn next_device(&mut self) {
        if !self.available_devices.is_empty() {
            self.selected_device_index = (self.selected_device_index + 1) % self.available_devices.len();
        }
    }

    
    pub fn prev_device(&mut self) {
        if !self.available_devices.is_empty() {
            self.selected_device_index = if self.selected_device_index == 0 {
                self.available_devices.len() - 1
            } else {
                self.selected_device_index - 1
            };
        }
    }

    
    pub fn toggle_capture(&mut self) {
        if self.is_capturing {
            self.stop_capture();
        } else {
            self.start_capture();
        }
    }

    pub fn increase_input_gain(&mut self) {
        self.adjust_input_gain(Self::INPUT_GAIN_STEP);
    }

    pub fn decrease_input_gain(&mut self) {
        self.adjust_input_gain(-Self::INPUT_GAIN_STEP);
    }

    
    pub fn start_capture(&mut self) {
        self.error_message = None;
        
        let device_name = self.selected_device().map(|s| s.to_string());
        
        match self.try_start_capture(device_name.as_deref()) {
            Ok(()) => {
                self.is_capturing = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to start: {}", e));
                self.is_capturing = false;
            }
        }
    }

    fn try_start_capture(&mut self, device_name: Option<&str>) -> Result<()> {
        let config = CaptureConfig {
            buffer_size: 1024,
            input_gain: self.input_gain,
        };
        let mut capture = AudioCapture::new(device_name, Some(config))?;
        
        self.sample_rate = capture.sample_rate();   
        let rx = capture.start()?;
        
        self.capture = Some(capture);
        self.sample_rx = Some(rx);
        
        Ok(())
    }

    fn adjust_input_gain(&mut self, delta: f32) {
        let new_gain = (self.input_gain + delta)
            .clamp(Self::INPUT_GAIN_MIN, Self::INPUT_GAIN_MAX);
        if (new_gain - self.input_gain).abs() < f32::EPSILON {
            return;
        }

        self.input_gain = new_gain;
        if self.is_capturing {
            self.stop_capture();
            self.start_capture();
        }
    }

    
    pub fn stop_capture(&mut self) {
        if let Some(mut capture) = self.capture.take() {
            capture.stop();
        }
        self.sample_rx = None;
        self.is_capturing = false;
        self.current_level = 0.0;
        self.peak_level = 0.0;
        self.waveform_buffer.clear();
        self.last_smooth = 0.0;
        self.has_last_smooth = false;
        self.raw_buffer.clear();
        self.last_freq_smooth = 0.0;
        self.has_last_freq_smooth = false;
        self.current_frequency = None;
        self.current_clarity = None;
    }

    
    
    pub fn update(&mut self) {
        
        self.peak_level *= 0.95;
        
        
        if let Some(rx) = self.sample_rx.take() {
            while let Ok(samples) = rx.try_recv() {
                let rms = Self::calculate_rms(&samples);
                self.current_level = rms;

                self.push_raw_samples(&samples);
                self.push_waveform_samples(&samples);

                if rms > self.peak_level {
                    self.peak_level = rms;
                }
            }

            self.sample_rx = Some(rx);
        }

        if self.raw_buffer.len() >= Self::DETECTOR_SIZE {
            let start = self.raw_buffer.len() - Self::DETECTOR_SIZE;
            let window = &self.raw_buffer[start..];
            if let Some((freq, clarity)) =
                self.frequency_detector.detect(window, self.sample_rate)
            {
                let alpha = Self::FREQUENCY_SMOOTH_ALPHA;
                let smoothed = if self.has_last_freq_smooth {
                    alpha * freq + (1.0 - alpha) * self.last_freq_smooth
                } else {
                    self.has_last_freq_smooth = true;
                    freq
                };
                self.last_freq_smooth = smoothed;
                self.current_frequency = Some(smoothed);
                self.current_clarity = Some(clarity);
            }
        }
        
        
        if self.sample_rx.is_none() || !self.is_capturing {
            self.current_level *= 0.9;
        }
    }

    
    
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();
        
        
        (rms * 3.0).clamp(0.0, 1.0)
    }

    fn raw_buffer_capacity() -> usize {
        Self::DETECTOR_SIZE * 4
    }

    fn detector_padding() -> usize {
        Self::DETECTOR_SIZE / 2
    }

    fn push_waveform_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        let decimation = Self::WAVEFORM_DECIMATION;
        let alpha = Self::WAVEFORM_SMOOTH_ALPHA;

        let display_gain = Self::WAVEFORM_DISPLAY_GAIN;
        for sample in samples.iter().step_by(decimation) {
            let boosted = (sample * display_gain).clamp(-1.0, 1.0);
            let smoothed = if self.has_last_smooth {
                alpha * boosted + (1.0 - alpha) * self.last_smooth
            } else {
                self.has_last_smooth = true;
                boosted
            };
            self.last_smooth = smoothed;
            self.waveform_buffer.push(smoothed);
        }

        let capacity = Self::WAVEFORM_CAPACITY;
        if self.waveform_buffer.len() > capacity {
            let overflow = self.waveform_buffer.len() - capacity;
            self.waveform_buffer.drain(0..overflow);
        }
    }

    fn push_raw_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        self.raw_buffer.extend_from_slice(samples);
        let capacity = Self::raw_buffer_capacity();
        if self.raw_buffer.len() > capacity {
            let overflow = self.raw_buffer.len() - capacity;
            self.raw_buffer.drain(0..overflow);
        }
    }
}

impl Default for TunerState {
    fn default() -> Self {
        Self::new()
    }
}
