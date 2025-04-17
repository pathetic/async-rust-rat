use std::os::windows::process::CommandExt;

pub fn system_commands(command: &str) {
    const HIDE: u32 = 0x08000000;
    match command {
        "shutdown" => {
            std::process::Command
                ::new("shutdown")
                .creation_flags(HIDE)
                .args(["/s", "/t", "0"])
                .spawn()
                .expect("Failed to shutdown");
        }
        "logout" => {
            std::process::Command
                ::new("shutdown")
                .creation_flags(HIDE)
                .args(["/l"])
                .spawn()
                .expect("Failed to log off");
        }
        "restart" => {
            std::process::Command
                ::new("shutdown")
                .creation_flags(HIDE)
                .args(["/r", "/t", "0"])
                .spawn()
                .expect("Failed to restart");
        }
        _ => {}
    }
}