use std::{
    fs,
    io::{BufRead, BufReader},
};

use crate::task::is_pid;

pub struct Process {
    pub pid: String,
    pub name: String,
    pub status: String,
    pub cpu_usage: f64,
    pub memory: usize,
}

pub fn get_cpu_total_idle() -> usize {
    let stat_details = fs::read_to_string("/proc/stat").unwrap();
    let line = stat_details.lines().next().unwrap();
    let stat: Vec<&str> = line[3..].split_whitespace().collect();
    let total = stat.iter().map(|num| num.parse::<usize>().unwrap()).sum();
    total
}

pub fn get_process() -> Vec<Process> {
    let mut processes: Vec<Process> = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let mut name = String::new();
            let mut memory: usize = 0;
            let mut status = String::new();
            let mut cpu_usage: f64 = 0.0;
            let filename = entry.file_name();
            let pid = filename.to_string_lossy();
            if !is_pid(&pid) {
                continue;
            }
            let stat_path = format!("/proc/{}/stat", pid);

            if let Ok(content) = fs::read_to_string(stat_path) {
                let start = content.find(")").unwrap();
                let values = &content[start + 1..].trim();
                let parts: Vec<&str> = values.split_whitespace().collect();
                status = parts[0].to_string();
                let cpu_time =
                    parts[11].parse::<usize>().unwrap() + parts[12].parse::<usize>().unwrap();
                cpu_usage = (cpu_time as f64 / get_cpu_total_idle() as f64) * 100.0;
            }

            let status_path = format!("/proc/{}/status", pid);

            if let Ok(status_file) = fs::File::open(status_path) {
                let reader = BufReader::new(status_file);
                for line in reader.lines() {
                    let content = line.unwrap();
                    let parts: Vec<&str> = content.split_whitespace().collect();
                    match parts[0] {
                        "Name:" => name = parts[1].to_string(),
                        "VmRSS:" => memory = parts[1].parse::<usize>().unwrap(),
                        _ => {}
                    }
                }
            }
            let ps1 = Process {
                pid: pid.to_string(),
                name,
                status,
                memory,
                cpu_usage,
            };
            processes.push(ps1);
        }
    }
    processes
}
