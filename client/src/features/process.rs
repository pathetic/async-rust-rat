use sysinfo::{ System, Pid };
use common::packets::{ ProcessList, Process };

pub fn process_list() -> ProcessList {
    let mut s = System::new_all();

    s.refresh_all();

    let mut process_list = ProcessList {
        processes: Vec::new(),
    };

    for (pid, process) in s.processes() {
        let process: Process = Process {
            pid: pid.as_u32() as usize,
            name: process.name().to_string(),
        };

        process_list.processes.push(process);
    }

    process_list
}

pub fn kill_process(pid: usize) {
    let mut s = System::new_all();

    s.refresh_all();

    if let Some(process) = s.process(Pid::from(pid)) {
        process.kill();
    }
}