//! Tuner state management for audio capture and pitch detection.

use anyhow::Result;
use crossbeam_channel::Receiver;

use crate::audio::capture::{AudioCapture, CaptureConfig, list_input_devices};

/// State for the tuner tab.
pub struct TunerState {
    /// Available input devices
    available_devices: Vec<String>,
    /// Currently selected device index
    selected_device_index: usize,
    /// Audio capture handle (None if not started)
    capture: Option<AudioCapture>,
    /// Sample receiver (None if not capturing)
    sample_rx: Option<Receiver<Vec<f32>>>,
    /// Current RMS level (0.0 to 1.0) for volume meter
    current_level: f32,
    /// Peak level with decay for visual effect
    peak_level: f32,
    /// Sample rate from the capture device
    sample_rate: u32,
    /// Whether capture is currently active
    is_capturing: bool,
    /// Error message if something went wrong
    error_message: Option<String>,
}

impl TunerState {
    /// Creates a new tuner state and enumerates available devices.
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
        }
    }

    /// Returns the list of available input devices.
    pub fn available_devices(&self) -> &[String] {
        &self.available_devices
    }

    /// Returns the currently selected device name.
    pub fn selected_device(&self) -> Option<&str> {
        self.available_devices.get(self.selected_device_index).map(|s| s.as_str())
    }

    /// Returns the selected device index.
    pub fn selected_device_index(&self) -> usize {
        self.selected_device_index
    }

    /// Returns the current RMS level (0.0 to 1.0).
    pub fn current_level(&self) -> f32 {
        self.current_level
    }

    /// Returns the peak level with decay.
    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }

    /// Returns the sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns whether capture is currently active.
    pub fn is_capturing(&self) -> bool {
        self.is_capturing
    }

    /// Returns any error message.
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Refresh the list of available devices.
    pub fn refresh_devices(&mut self) {
        self.available_devices = list_input_devices().unwrap_or_default();
        if self.selected_device_index >= self.available_devices.len() {
            self.selected_device_index = 0;
        }
    }

    /// Select the next device in the list.
    pub fn next_device(&mut self) {
        if !self.available_devices.is_empty() {
            self.selected_device_index = (self.selected_device_index + 1) % self.available_devices.len();
        }
    }

    /// Select the previous device in the list.
    pub fn prev_device(&mut self) {
        if !self.available_devices.is_empty() {
            self.selected_device_index = if self.selected_device_index == 0 {
                self.available_devices.len() - 1
            } else {
                self.selected_device_index - 1
            };
        }
    }

    /// Toggle audio capture on/off.
    pub fn toggle_capture(&mut self) {
        if self.is_capturing {
            self.stop_capture();
        } else {
            self.start_capture();
        }
    }

    /// Start audio capture on the selected device.
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
        let config = CaptureConfig { buffer_size: 2048 };
        let mut capture = AudioCapture::new(device_name, Some(config))?;
        
        self.sample_rate = capture.sample_rate();
        let rx = capture.start()?;
        
        self.capture = Some(capture);
        self.sample_rx = Some(rx);
        
        Ok(())
    }

    /// Stop audio capture.
    pub fn stop_capture(&mut self) {
        if let Some(mut capture) = self.capture.take() {
            capture.stop();
        }
        self.sample_rx = None;
        self.is_capturing = false;
        self.current_level = 0.0;
        self.peak_level = 0.0;
    }

    /// Update the tuner state - call this each frame.
    /// Processes any pending audio samples and updates levels.
    pub fn update(&mut self) {
        // Decay the peak level
        self.peak_level *= 0.95;
        
        // Process samples if we have a receiver
        if let Some(rx) = &self.sample_rx {
            // Get the latest samples (non-blocking)
            while let Ok(samples) = rx.try_recv() {
                // Calculate RMS level
                let rms = Self::calculate_rms(&samples);
                self.current_level = rms;
                
                // Update peak with the new level
                if rms > self.peak_level {
                    self.peak_level = rms;
                }
            }
        }
        
        // Decay current level when not receiving
        if self.sample_rx.is_none() || !self.is_capturing {
            self.current_level *= 0.9;
        }
    }

    /// Calculate RMS (Root Mean Square) of samples.
    /// Returns a value from 0.0 to 1.0 (clamped).
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();
        
        // Scale up and clamp - typical audio RMS is quite low
        (rms * 3.0).clamp(0.0, 1.0)
    }
}

impl Default for TunerState {
    fn default() -> Self {
        Self::new()
    }
}
