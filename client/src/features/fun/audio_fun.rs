#[cfg(windows)]
mod imp {
    use winapi::um::winuser::{
        KEYEVENTF_KEYUP, keybd_event,
        VK_VOLUME_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE
    };
    use std::process::Command;
    use std::thread;
    use winapi::um::utilapiset::Beep;

    pub fn beep(freq_duration: &str) {
        let freq_duration = freq_duration.split(":").collect::<Vec<&str>>();
        let freq = freq_duration[0].parse::<u32>().unwrap();
        let duration = freq_duration[1].parse::<u32>().unwrap();
        unsafe { Beep(freq, duration); }
    }

    pub fn toggle_volume_mute() {
        unsafe {
            keybd_event(VK_VOLUME_MUTE as u8, 0, 0, 0);
            keybd_event(VK_VOLUME_MUTE as u8, 0, KEYEVENTF_KEYUP, 0);
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub enum Volume {
        Max,
        Min,
    }

    pub fn set_volume(volume: Volume) {
        toggle_volume_mute();
        thread::sleep(std::time::Duration::from_millis(100));
        let key = match volume {
            Volume::Max => VK_VOLUME_UP,
            Volume::Min => VK_VOLUME_DOWN,
        };
        unsafe {
            for _ in 0..50 {
                keybd_event(key as u8, 0, 0, 0);
                keybd_event(key as u8, 0, KEYEVENTF_KEYUP, 0);
                thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }

    pub fn speak_text(text: &str) {    
        let ps_text = text.replace("\"", "\\\""); // Escape quotes for PowerShell
        let ps_script = format!(
            "Add-Type -AssemblyName System.Speech; \
             $speak = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
             $speak.Speak(\"{}\")", 
            ps_text
        );
        match Command::new("powershell.exe")
            .args(["-Command", &ps_script])
            .spawn() {
            Ok(_) => {},
            Err(_e) => {
                speak_text_with_sapi(text);
            }
        }
    }

    pub fn speak_text_with_sapi(text: &str) {
        let vbs_text = text.replace("\"", "\"\""); // Escape quotes for VBScript
        let vbs_script = format!(
            "CreateObject(\"SAPI.SpVoice\").Speak \"{}\"", 
            vbs_text
        );
        match Command::new("cmd.exe")
            .args(["/c", "echo", &vbs_script, ">", "temp_speech.vbs", "&&", "cscript", "//nologo", "temp_speech.vbs", "&&", "del", "temp_speech.vbs"])
            .spawn() {
            Ok(_) => {},
            Err(_e) => {},
        }
    }

    pub fn key_to_midi_to_freq(key: u32) -> Option<u32> {
        let midi = match key {
            1 => Some(71), // B4
            2 => Some(70), // A#4
            3 => Some(69), // A4 (440Hz)
            4 => Some(68), // G#4
            5 => Some(67), // G4
            6 => Some(66), // F#4
            7 => Some(65), // F4
            8 => Some(64), // E4
            9 => Some(63), // D#4
            10 => Some(62), // D4
            11 => Some(61), // C#4
            12 => Some(60), // C4 (Middle C)
            _ => None,
        };
        if let Some(midi) = midi {
            let freq = 440.0 * 2f64.powf((midi as f64 - 69.0) / 12.0);
            Some(freq.round() as u32)
        } else {
            None
        }
    }

    pub fn piano_key(key: u32) {
        if let Some(freq) = key_to_midi_to_freq(key) {
            unsafe {
                Beep(freq, 300);
            }
        }
    }
}

#[cfg(unix)]
mod imp {
    pub fn beep(_freq_duration: &str) {}
    pub fn toggle_volume_mute() {}
    pub enum Volume { Max, Min }
    pub fn set_volume(_volume: Volume) {}
    pub fn speak_text(_text: &str) {}
    pub fn speak_text_with_sapi(_text: &str) {}
    pub fn key_to_midi_to_freq(_key: u32) -> Option<u32> { None }
    pub fn piano_key(_key: u32) {}
}

pub use imp::*;