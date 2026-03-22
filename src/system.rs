use std::fs;

pub struct CpuState {
    id: String,
    idle: usize,
    total: usize,
}

pub struct CpuUsage {
    pub id: String,
    pub usage: f64,
}

pub fn get_cpu_stat() -> Vec<CpuState> {
    let cpu_details = fs::read_to_string("/proc/stat").unwrap();
    let mut cpus: Vec<CpuState> = Vec::new();
    for line in cpu_details.lines() {
        if line.starts_with("cpu") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let id = parts[0].to_string();
            let values: Vec<usize> = parts[1..]
                .iter()
                .filter_map(|value| value.parse::<usize>().ok())
                .collect();
            let idle = values[3] + values[4];
            let total: usize = values.iter().sum();
            cpus.push(CpuState { id, idle, total });
        }
    }
    cpus
}

pub fn get_cpu_usage(previous: &Vec<CpuState>, current: &Vec<CpuState>) -> Vec<CpuUsage> {
    let mut cpus: Vec<CpuUsage> = Vec::new();

    for (prev, curr) in previous.iter().zip(current.iter()) {
        let idle_delta = curr.idle - prev.idle;
        let total_delta = curr.total - prev.total;
        let id = curr.id.to_string();

        let usage = if total_delta == 0 {
            0.0
        } else {
            (1.0 - idle_delta as f64 / total_delta as f64) * 100.0
        };
        cpus.push(CpuUsage { id, usage });
    }
    cpus
}

pub fn get_uptime() -> String {
    let read_uptime = fs::read_to_string("/proc/uptime").unwrap();
    let uptime_stat: Vec<&str> = read_uptime.split(".").collect();
    let uptime: usize = uptime_stat[0].parse::<usize>().unwrap();

    let uptime_format = format!("{}:{}:{}", uptime / 3600, (uptime % 3600) / 60, uptime % 60);
    uptime_format
}

pub fn get_memory() -> (usize, usize, usize, usize, usize) {
    let mem_details = fs::read_to_string("/proc/meminfo").unwrap();
    let mut total: usize = 0;
    let mut free: usize = 0;
    let mut cached: usize = 0;
    let mut bufferes: usize = 0;
    for line in mem_details.lines() {
        let content: Vec<&str> = line.split_whitespace().collect();
        match content[0].trim() {
            "MemTotal:" => total = content[1].trim().parse::<usize>().unwrap(),
            "MemFree:" => free = content[1].trim().parse::<usize>().unwrap(),
            "Buffers:" => bufferes = content[1].trim().parse::<usize>().unwrap(),
            "Cached:" => cached = content[1].trim().parse::<usize>().unwrap(),
            _ => {}
        }
    }
    let used = total - free - bufferes - cached;
    (total, used, free, bufferes, cached)
}

pub fn get_swap() -> Option<(usize, usize)> {
    if let Ok(swap_details) = fs::read_to_string("/proc/swaps") {
        let line: Vec<&str> = swap_details.lines().collect();
        let content: Vec<&str> = line[1].split_whitespace().collect();
        let total = content[2].parse::<usize>().unwrap();
        let used = content[3].parse::<usize>().unwrap();
        Some((total, used))
    } else {
        None
    }
}
