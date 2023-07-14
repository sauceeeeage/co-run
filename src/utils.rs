use rand::Rng;
use std::time::Duration;

#[allow(dead_code)]
pub fn get_rand_in_range(range: (i32, i32)) -> i32 {
    let (start, end) = range;
    rand::thread_rng().gen_range(start..=end)
}

pub fn get_cpu_info() -> usize {
    let stdout = std::process::Command::new("nproc")
        .arg("--all")
        .output()
        .expect("failed to get cpuinfo from nproc");
    let s = std::str::from_utf8(&stdout.stdout);
    let s = s
        .unwrap_or_else(|_| panic!("nproc did not return"))
        .to_string();
    let s = s.trim().to_string();
    s.parse::<usize>().expect("failed to parse nproc output")
}

pub fn get_duration(input: Option<f64>) -> Duration {
    let duration = input.unwrap_or_else(|| panic!("duration is not specified"));
    Duration::from_secs((duration * 3600.) as u64)
}

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
