use std::os::windows::process::CommandExt;

const HIDE: u32 = 0x08000000;

pub fn system_commands(command: &str) {
    
    match command {
        "shutdown" => run_command("shutdown", &["/s", "/t", "0"]),
        "logout" => run_command("shutdown", &["/l"]),
        "restart" => run_command("shutdown", &["/r", "/t", "0"]),
        _ => {}
    }
}

pub fn run_command(command: &str, args: &[&str]) {
    let _ = std::process::Command
        ::new(command)
        .creation_flags(HIDE)
        .args(args)
        .spawn()
        .expect("Failed to run command").wait();
}