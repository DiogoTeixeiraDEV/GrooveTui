use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleRate, Stream, StreamConfig};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const DEFAULT_BUFFER_SIZE: usize = 2048;

pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices: Vec<String> = host
        .input_devices()
        .context("Failed to enumerate input devices")?
        .filter_map(|d| d.name().ok())
        .collect();

    Ok(devices)
}


pub fn default_input_device_name() -> Result<String> {
    let host = cpal::default_host();
    host.default_input_device()
        .ok_or_else(|| anyhow!("No default input device available"))?
        .name()
        .context("Failed to get default device name")
}


#[derive(Clone, Debug)]
pub struct CaptureConfig {
    
    
    pub buffer_size: usize,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

pub struct AudioCapture {
    device: Device,
    config: StreamConfig,
    capture_config: CaptureConfig,
    stream: Option<Stream>,
    is_running: Arc<AtomicBool>,
}

impl AudioCapture {
    
    
    pub fn new(device_name: Option<&str>, capture_config: Option<CaptureConfig>) -> Result<Self> {
        let host = cpal::default_host();
        let device = Self::find_device(&host, device_name)?;
        let config = Self::get_stream_config(&device)?;
        let capture_config = capture_config.unwrap_or_default();

        Ok(Self {
            device,
            config,
            capture_config,
            stream: None,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    
    pub fn channels(&self) -> u16 {
        self.config.channels
    }

    
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn start(&mut self) -> Result<Receiver<Vec<f32>>> {
        if self.is_running() {
            return Err(anyhow!("Capture is already running"));
        }

        let (tx, rx) = bounded::<Vec<f32>>(1);
        let channels = self.config.channels as usize;
        let buffer_size = self.capture_config.buffer_size;
        let is_running = Arc::clone(&self.is_running);

        
        let mut sample_buffer: Vec<f32> = Vec::with_capacity(buffer_size);

        let stream = self
            .device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    Self::process_samples(data, channels, &mut sample_buffer, buffer_size, &tx);
                },
                move |err| {
                    
                    eprintln!("Audio capture error: {}", err);
                    is_running.store(false, Ordering::SeqCst);
                },
                None, 
            )
            .context("Failed to build input stream")?;

        stream.play().context("Failed to start audio stream")?;

        self.stream = Some(stream);
        self.is_running.store(true, Ordering::SeqCst);

        Ok(rx)
    }

    
    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
        }
        self.is_running.store(false, Ordering::SeqCst);
    }

    

    fn find_device(host: &Host, device_name: Option<&str>) -> Result<Device> {
        match device_name {
            Some(name) => {
                let devices = host
                    .input_devices()
                    .context("Failed to enumerate input devices")?;

                for device in devices {
                    if let Ok(dev_name) = device.name() {
                        if dev_name == name {
                            return Ok(device);
                        }
                    }
                }
                Err(anyhow!("Input device '{}' not found", name))
            }
            None => host
                .default_input_device()
                .ok_or_else(|| anyhow!("No default input device available")),
        }
    }

    fn get_stream_config(device: &Device) -> Result<StreamConfig> {
        let supported_config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        
        Ok(StreamConfig {
            channels: supported_config.channels(),
            sample_rate: supported_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        })
    }

    fn process_samples(
        data: &[f32],
        channels: usize,
        buffer: &mut Vec<f32>,
        target_size: usize,
        tx: &Sender<Vec<f32>>,
    ) {
        
        for chunk in data.chunks(channels) {
            if let Some(&sample) = chunk.first() {
                buffer.push(sample);

                
                if buffer.len() >= target_size {
                    
                    
                    let samples = std::mem::replace(buffer, Vec::with_capacity(target_size));
                    let _ = tx.try_send(samples);
                }
            }
        }
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices_does_not_panic() {
        
        let _ = list_input_devices();
    }

    #[test]
    fn test_default_config() {
        let config = CaptureConfig::default();
        assert_eq!(config.buffer_size, DEFAULT_BUFFER_SIZE);
    }
}
