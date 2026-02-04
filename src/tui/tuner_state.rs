

use anyhow::Result;
use crossbeam_channel::Receiver;

use crate::audio::capture::{AudioCapture, CaptureConfig, list_input_devices};
use crate::audio::frequency::FrequencyDetector;


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
}

impl TunerState {
    
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
            waveform_buffer: Vec::with_capacity(Self::waveform_capacity()),
            last_smooth: 0.0,
            has_last_smooth: false,
            raw_buffer: Vec::with_capacity(Self::raw_buffer_capacity()),
            last_freq_smooth: 0.0,
            has_last_freq_smooth: false,
            frequency_detector: FrequencyDetector::new(
                Self::detector_size(),
                Self::detector_padding(),
                Self::power_threshold(),
                Self::clarity_threshold(),
            ),
            current_frequency: None,
            current_clarity: None,
        }
    }

    
    pub fn available_devices(&self) -> &[String] {
        &self.available_devices
    }

    
    pub fn selected_device(&self) -> Option<&str> {
        self.available_devices.get(self.selected_device_index).map(|s| s.as_str())
    }

    
    pub fn selected_device_index(&self) -> usize {
        self.selected_device_index
    }

    
    pub fn current_level(&self) -> f32 {
        self.current_level
    }

    
    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }

    
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
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

    pub fn current_clarity(&self) -> Option<f32> {
        self.current_clarity
    }

    pub fn waveform_samples(&self) -> &[f32] {
        &self.waveform_buffer
    }

    
    pub fn refresh_devices(&mut self) {
        self.available_devices = list_input_devices().unwrap_or_default();
        if self.selected_device_index >= self.available_devices.len() {
            self.selected_device_index = 0;
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
        let config = CaptureConfig { buffer_size: 1024 };
        let mut capture = AudioCapture::new(device_name, Some(config))?;
        
        self.sample_rate = capture.sample_rate();   
        let rx = capture.start()?;
        
        self.capture = Some(capture);
        self.sample_rx = Some(rx);
        
        Ok(())
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

        if self.raw_buffer.len() >= Self::detector_size() {
            let start = self.raw_buffer.len() - Self::detector_size();
            let window = &self.raw_buffer[start..];
            if let Some((freq, clarity)) =
                self.frequency_detector.detect(window, self.sample_rate)
            {
                let alpha = Self::frequency_smooth_alpha();
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

    fn waveform_capacity() -> usize {
        256
    }

    fn raw_buffer_capacity() -> usize {
        Self::detector_size() * 4
    }

    fn detector_size() -> usize {
        1024
    }

    fn detector_padding() -> usize {
        Self::detector_size() / 2
    }

    fn power_threshold() -> f64 {
        5.0
    }

    fn clarity_threshold() -> f64 {
        0.7
    }

    fn frequency_smooth_alpha() -> f32 {
        0.25
    }

    fn waveform_decimation() -> usize {
        4
    }

    fn smooth_alpha() -> f32 {
        0.2
    }

    fn push_waveform_samples(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }

        let decimation = Self::waveform_decimation();
        let alpha = Self::smooth_alpha();

        for sample in samples.iter().step_by(decimation) {
            let smoothed = if self.has_last_smooth {
                alpha * sample + (1.0 - alpha) * self.last_smooth
            } else {
                self.has_last_smooth = true;
                *sample
            };
            self.last_smooth = smoothed;
            self.waveform_buffer.push(smoothed);
        }

        let capacity = Self::waveform_capacity();
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
