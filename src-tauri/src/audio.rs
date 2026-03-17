use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Audio format expected by Whisper: 16kHz, mono, f32
pub const SAMPLE_RATE: u32 = 16000;

/// Shared audio buffer for collecting samples
pub type AudioBuffer = Arc<Mutex<Vec<f32>>>;

/// Audio capture state (Send + Sync compatible)
pub struct AudioCapture {
    mic_buffer: AudioBuffer,
    system_buffer: AudioBuffer,
    is_capturing: Arc<Mutex<bool>>,
    stop_flag: Arc<Mutex<bool>>,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            mic_buffer: Arc::new(Mutex::new(Vec::new())),
            system_buffer: Arc::new(Mutex::new(Vec::new())),
            is_capturing: Arc::new(Mutex::new(false)),
            stop_flag: Arc::new(Mutex::new(false)),
        }
    }

    /// Start capturing microphone audio (runs in separate thread)
    pub fn start_microphone(&self) -> Result<()> {
        let buffer = self.mic_buffer.clone();
        let is_capturing = self.is_capturing.clone();
        let stop_flag = self.stop_flag.clone();

        // Reset stop flag
        *stop_flag.lock().unwrap() = false;

        thread::spawn(move || {
            if let Err(e) = run_audio_capture(buffer, stop_flag, true) {
                log::error!("Microphone capture error: {}", e);
            }
            *is_capturing.lock().unwrap() = false;
        });

        *self.is_capturing.lock().unwrap() = true;
        log::info!("Microphone capture started");
        Ok(())
    }

    /// Start capturing system audio
    pub fn start_system_audio(&self) -> Result<()> {
        let buffer = self.system_buffer.clone();
        let stop_flag = self.stop_flag.clone();

        *stop_flag.lock().unwrap() = false;

        thread::spawn(move || {
            if let Err(e) = run_audio_capture(buffer, stop_flag, false) {
                log::error!("System audio capture error: {}", e);
            }
        });

        log::info!("System audio capture started");
        Ok(())
    }

    /// Stop all audio capture
    pub fn stop(&self) {
        *self.stop_flag.lock().unwrap() = true;
        *self.is_capturing.lock().unwrap() = false;
        log::info!("Audio capture stop requested");
    }

    /// Check if currently capturing
    pub fn is_capturing(&self) -> bool {
        *self.is_capturing.lock().unwrap()
    }

    /// Get mixed audio buffer (mic + system) and clear it
    pub fn take_audio(&self) -> Vec<f32> {
        let mut mic = self.mic_buffer.lock().unwrap();
        let mut sys = self.system_buffer.lock().unwrap();

        let max_len = mic.len().max(sys.len());
        let mut mixed = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let mic_sample = mic.get(i).copied().unwrap_or(0.0);
            let sys_sample = sys.get(i).copied().unwrap_or(0.0);
            let mixed_sample = (mic_sample + sys_sample).clamp(-1.0, 1.0);
            mixed.push(mixed_sample);
        }

        mic.clear();
        sys.clear();
        mixed
    }

    /// Take only microphone audio
    pub fn take_mic_audio(&self) -> Vec<f32> {
        let mut buf = self.mic_buffer.lock().unwrap();
        let audio = buf.clone();
        buf.clear();
        audio
    }

    /// Get current buffer duration in seconds
    pub fn buffer_duration(&self) -> f32 {
        let mic = self.mic_buffer.lock().unwrap();
        mic.len() as f32 / SAMPLE_RATE as f32
    }
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

/// Run audio capture in a thread
fn run_audio_capture(
    buffer: AudioBuffer,
    stop_flag: Arc<Mutex<bool>>,
    use_mic: bool,
) -> Result<()> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();

    let device = if use_mic {
        host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?
    } else {
        // Try to find loopback device
        find_loopback_device(&host)?
    };

    log::info!("Using audio device: {}", device.name().unwrap_or_default());

    let config = device
        .default_input_config()
        .map_err(|e| anyhow::anyhow!("Failed to get input config: {}", e))?;

    let channels = config.channels() as usize;

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), buffer, channels)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), buffer, channels)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), buffer, channels)?,
        _ => return Err(anyhow::anyhow!("Unsupported sample format")),
    };

    stream.play()?;

    // Wait until stop flag is set
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if *stop_flag.lock().unwrap() {
            break;
        }
    }

    drop(stream);
    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: AudioBuffer,
    channels: usize,
) -> Result<cpal::Stream>
where
    T: cpal::Sample + cpal::SizedSample + Into<f32>,
{
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut buf = buffer.lock().unwrap();

            for chunk in data.chunks(channels) {
                let mono: f32 = if channels == 1 {
                    chunk[0].into()
                } else {
                    let sum: f32 = chunk.iter().map(|s| -> f32 { (*s).into() }).sum();
                    sum / channels as f32
                };
                buf.push(mono);
            }

            // Keep buffer at reasonable size (max 60 seconds)
            let max_samples = SAMPLE_RATE as usize * 60;
            if buf.len() > max_samples {
                let drain_count = buf.len() - max_samples;
                buf.drain(0..drain_count);
            }
        },
        |err| log::error!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn find_loopback_device(host: &cpal::Host) -> Result<cpal::Device> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let devices = host.input_devices()?;

    for device in devices {
        let name = device.name().unwrap_or_default();
        if name.contains("monitor") || name.contains("loopback") || name.contains("Loopback") {
            return Ok(device);
        }
    }

    host.default_output_device()
        .ok_or_else(|| anyhow::anyhow!(
            "No loopback device found. On Linux, enable PulseAudio monitor:\n\
             pactl load-module module-null-sink sink_name=virtual\n\
             pactl load-module module-remap-source master=virtual.monitor source_name=virtual_mic"
        ))
}

pub fn list_devices() -> Result<Vec<String>> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let mut devices = Vec::new();

    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                devices.push(format!("input: {}", name));
            }
        }
    }

    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                devices.push(format!("output: {}", name));
            }
        }
    }

    Ok(devices)
}

/// Convert f32 audio samples to WAV bytes for Whisper API
pub fn samples_to_wav(samples: &[f32]) -> Vec<u8> {
    let sample_rate = SAMPLE_RATE;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = samples.len() as u32 * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::new();

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Convert f32 to i16
    for &sample in samples {
        let i16_sample = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&i16_sample.to_le_bytes());
    }

    wav
}
