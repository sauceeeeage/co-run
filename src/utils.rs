use rand::Rng;
use regex::Regex;
use std::{path::PathBuf, time::Duration};

#[allow(dead_code)]
#[inline(always)]
pub fn get_rand_in_range(range: (i32, i32)) -> i32 {
    let (start, end) = range;
    rand::thread_rng().gen_range(start..=end)
}

pub fn get_cpu_info() -> usize {
    let stdout;
    if cfg!(target_os = "linux") {
        stdout = std::process::Command::new("nproc")
            .arg("--all")
            .output()
            .expect("failed to get cpuinfo from nproc");
    } else if cfg!(target_os = "macos") {
        stdout = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.ncpu")
            .output()
            .expect("failed to get cpuinfo from sysctl");
    } else if cfg!(target_os = "windows") {
        stdout = std::process::Command::new("wmic")
            .arg("cpu")
            .arg("get")
            .arg("NumberOfCores")
            .output()
            .expect("failed to get cpuinfo from wmic");
    } else {
        panic!("unsupported OS");
    }

    let s = std::str::from_utf8(&stdout.stdout)
        .unwrap_or_else(|_| panic!("failed to get cpuinfo from stdout"))
        .trim();
    let re = Regex::new(r"[0-9]+")
        .unwrap_or_else(|_| panic!("failed to get the cores count of the CPU"));
    let result = re.captures(s).unwrap();

    result
        .get(0)
        .unwrap()
        .as_str()
        .parse::<usize>()
        .expect("failed to parse cpuinfo output")
}

#[inline(always)]
pub fn get_duration(input: Option<f64>) -> Duration {
    let duration = input.unwrap_or_else(|| panic!("duration is not specified"));
    Duration::from_secs((duration * 3600.) as u64)
}

#[inline(always)]
pub fn get_range(range: Option<String>) -> (i32, i32) {
    let range = range.unwrap_or_else(|| panic!("range is not specified"));
    let mut range = range.split('-');
    let start = range
        .next()
        .unwrap_or_else(|| panic!("start of the range is not specified"))
        .parse::<i32>()
        .unwrap_or_else(|_| panic!("start of the range is not an integer"));
    let end = range
        .next()
        .unwrap_or_else(|| panic!("end of the range is not specified"))
        .parse::<i32>()
        .unwrap_or_else(|_| panic!("end of the range is not an integer"));
    (start, end)
}

#[inline(always)]
pub fn delete_bin(file_path: PathBuf) {
    if file_path.exists() {
        std::fs::remove_file(file_path).expect("failed to delete binary file");
    }
}
