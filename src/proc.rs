use std::{
    fs,
    io::{BufRead, BufReader},
};

use crate::task::is_pid;

struct Process {
    pid: String,
    name: String,
    cpu: usize,
    memory: String,
}

pub fn get_process() {
    if let Ok(entries) = fs::read_dir("/proc") {
        let mut process: Vec<Process> = Vec::new();
        for entry in entries.flatten() {
            let mut name = String::new();
            let mut memory = String::new();
            let mut cpu: usize = 0;
            let filename = entry.file_name();
            let pid = filename.to_string_lossy();
            if !is_pid(&pid) {
                continue;
            }
            let stat_path = format!("/proc/{}/stat", pid);

            if let Ok(content) = fs::read_to_string(stat_path) {
                let parts: Vec<&str> = content.split_whitespace().collect();
                name = parts[1].to_string();
                cpu = parts[13].parse::<usize>().unwrap() + parts[14].parse::<usize>().unwrap();
            }

            let status_path = format!("/proc/{}/status", pid);

            if let Ok(status_file) = fs::File::open(status_path) {
                let reader = BufReader::new(status_file);
                for line in reader.lines() {
                    let content = line.unwrap();
                    if content.starts_with("VmRSS") {
                        let content_parts: Vec<&str> = content.split(":").collect();
                        memory = content_parts[1].trim().to_string();
                        break;
                    }
                }
            }
            let ps1 = Process {
                pid: pid.to_string(),
                name: name,
                memory: memory,
                cpu: cpu,
            };
            process.push(ps1);
        }
    }
}
