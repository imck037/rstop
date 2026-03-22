use std::{fs, io::Read};

pub fn is_pid(str: &str) -> bool {
    str.chars().all(|c| c.is_numeric())
}

fn count_threads(pid: &str) -> usize {
    let task_path = format!("/proc/{}/task", pid);

    match fs::read_dir(task_path) {
        Ok(entries) => entries.count(),
        Err(_) => 0,
    }
}

pub fn tasks() -> (usize, usize, usize, usize, usize, usize) {
    let mut total_process = 0;
    let mut running_process = 0;
    let mut sleeping_process = 0;
    let mut stopped_process = 0;
    let mut zombie_process = 0;
    let mut threads = 0;

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let pid = filename.to_string_lossy();
            if !is_pid(&pid) {
                continue;
            }
            total_process += 1;

            threads += count_threads(&pid);

            let mut contents = String::new();
            let stat_path = format!("/proc/{}/stat", pid);
            if let Ok(mut file) = fs::File::open(stat_path) {
                if file.read_to_string(&mut contents).is_ok() {
                    let parts: Vec<&str> = contents.split_whitespace().collect();

                    if parts.len() > 2 {
                        match parts[2] {
                            "S" => sleeping_process += 1,
                            "R" => running_process += 1,
                            "Z" => zombie_process += 1,
                            "T" => stopped_process += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    (
        total_process,
        running_process,
        sleeping_process,
        stopped_process,
        zombie_process,
        threads,
    )
}
