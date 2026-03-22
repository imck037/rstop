use std::fs;

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
