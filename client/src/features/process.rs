use sysinfo::{ System, Pid };
use common::packets::{ ProcessList, Process };
use std::process::Command;

use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{OpenThread, ResumeThread, SuspendThread};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, Thread32First, Thread32Next, THREADENTRY32};
use winapi::um::winnt::{HANDLE, THREAD_SUSPEND_RESUME};

pub fn process_list() -> ProcessList {
    let mut s = System::new_all();

    s.refresh_all();

    let mut process_list = ProcessList {
        processes: Vec::new(),
    };

    for (pid, process) in s.processes() {
        let process: Process = Process {
            pid: pid.as_u32() as usize,
            name: process.name().to_string_lossy().to_string(),
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

pub fn start_process(name: String) {
    Command::new(name).spawn().unwrap();
}

pub fn suspend_process(pid: usize) {
    let pid = pid as u32;
    unsafe {
        let te: &mut THREADENTRY32 = &mut std::mem::zeroed();
        (*te).dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

        let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);

        if Thread32First(snapshot, te) == 1 {
            loop {
                if pid == (*te).th32OwnerProcessID {
                    let tid = (*te).th32ThreadID;

                    let thread: HANDLE = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
                    let _ = SuspendThread(thread) as i32 == -1i32;
                    CloseHandle(thread);
                }

                if Thread32Next(snapshot, te) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
}

pub fn resume_process(pid: usize) {
    let pid = pid as u32;
    unsafe {
        let te: &mut THREADENTRY32 = &mut std::mem::zeroed();
        (*te).dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

        let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);

        if Thread32First(snapshot, te) == 1 {
            loop {
                if pid == (*te).th32OwnerProcessID {
                    let tid = (*te).th32ThreadID;

                    let thread: HANDLE = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
                    let _ = ResumeThread(thread) as i32 == -1i32;
                    CloseHandle(thread);
                }

                if Thread32Next(snapshot, te) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
}
