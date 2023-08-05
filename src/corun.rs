// use std::io::Write;
use std::path::PathBuf;
// use clap::error::ContextValue::String;
use std::string::String;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
// use std::sync::Mutex;
use std::thread::{self};
use std::time;
use threadpool::ThreadPool;

// use anyhow::Error;
use chrono::Utc;
use rand::Rng;
use regex::Regex;
use tracing::{debug, info, warn};

use crate::logging::{logging, Log};
use crate::utils::delete_bin;

// TODO: need logging(tracing and tracing-sub) on each program start time and finish time and the order thy ran(√), continuous scheduling, randomly schedule multiple programs(i actually didn't feel the need for this)
// TODO: need to add git submodule for polybench(√)
// FIXME: probably more helpful if we read in program paths and arguments from a file(√)
// TODO: do permutation on the program args and other programs, and then use them as program pools
// TODO: need to add back tracing for the program start time and finish time using logging in order to tell which programs are running at given time, thus can figure out the DMC for that given time(√)
// TODO: do GEMM(interval between 500-5000(rand num), run for 5 hr, night time(est)) polybench and get DMC(on cycle2.cs machine)
// TODO: make tmpdir for each compliation, and delete them after the program finishes to make sure each concurrent programs are running the correct program(use tmpfile crate, may also need to add more cli args to the Makefile)

// TODO: concurrently launch programs, instead of like right now(one by one)

static PROG_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub(crate) cmd: Option<String>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) range: Option<(i32, i32)>,
    pub(crate) args: Option<Vec<String>>,
}

fn get_prog_name(prog_path: Option<PathBuf>) -> String {
    let pn_re = Regex::new(r"\/[a-zA-Z]+\/$").unwrap();
    pn_re
        .captures(prog_path.as_ref().unwrap().to_str().unwrap())
        .unwrap()
        .get(0)
        .unwrap()
        .as_str()
        .replace('/', "")
        .to_string()
}

pub fn co_run(
    programs: Arc<RwLock<Vec<Program>>>,
    total_dur: time::Duration,
    cpu_cnt: usize,
    // log_file: &'static mut Arc<Mutex<Result<File, std::io::Error>>>,
) -> Vec<Log> {
    let human_readable = Arc::new(Mutex::new(Vec::new()));
    let mut pool = ThreadPool::new(cpu_cnt);
    let timer_start = time::Instant::now();
    pool.set_num_threads(cpu_cnt);
    // let log_file = Arc::new(Mutex::new(
    //     OpenOptions::new()
    //         .read(true)
    //         .write(true)
    //         .create(true)
    //         .open("log"),
    // ));
    while (timer_start.elapsed()) < total_dur {
        // let log_file = Arc::clone(&log_file);
        let programs = Arc::clone(&programs);
        let human_readable = Arc::clone(&human_readable);

        if pool.queued_count() == 0 {
            warn!("program queue is empty, adding new jobs");
            info!("current queue size: {}", pool.queued_count());
            info!("current active threads: {}", pool.active_count());
            pool.execute(move || {
                // FIXME: the problem maybe the lock issue, but i'm not sure
                // FIXME: give subroutines Arc instead of aquiring the lock here(let those subroutines get the locks)
                warn!("cpu is not fully utilized, launching more programs");
                // let tid: u64 = thread::current().id().as_u64().into();
                let c_programs = programs.read().unwrap();
                let current_prog = c_programs
                    [rand::thread_rng().gen_range(0..programs.read().unwrap().len())]
                .clone();
                let c = current_prog.clone();

                let mut log = Log {
                    start: Utc::now(),
                    finish: None,
                    duration: None,
                    prog_id: PROG_COUNTER.fetch_add(1, Ordering::Relaxed),
                    prog_name: get_prog_name(c.path),
                    cmd: c.cmd.clone(),
                    args: c.args.clone(),
                };
                // logging(0, log.clone(), human_readable, log_file);
                let full_log = single_run(&current_prog, &mut log);
                logging(1, full_log, human_readable);
                delete_bin(log.prog_id.clone().to_string().into());
            });
            // pool.join();
            // let sleep_dur = time::Duration::from_secs(1);
            // thread::sleep(sleep_dur); // FIXME: having some issues below, using this for now
        } else {
            warn!("program queue is not empty, sleep for 1 sec");
            let sleep_dur = time::Duration::from_secs(1);
            thread::sleep(sleep_dur); // FIXME: having some issues below, using this for now
                                      // pool.join();
        }
    }
    warn!("time's up! exiting...");
    let h = human_readable.lock().unwrap();
    h.to_vec()
}

#[inline(always)]
fn single_run(program: &Program, unfinished_log: &mut Log) -> Log {
    let file_name = unfinished_log.prog_id;
    let prog_path = program.path.clone();
    if !std::env::current_dir()
        .unwrap()
        .ends_with(program.path.as_ref().unwrap())
    {
        assert!(std::env::set_current_dir(prog_path.unwrap()).is_ok());
    }
    let mut fixed_args = Vec::new();
    for str in program.args.clone().as_mut().unwrap_or(&mut Vec::new()) {
        let re = Regex::new(r"\=\*").unwrap();
        let file_re = Regex::new(r"file_name\=\*").unwrap();
        let rand =
            rand::thread_rng().gen_range(program.range.unwrap().0..=program.range.unwrap().1);
        if file_re.is_match(str) {
            // debug!("file_name is matched");
            // debug!("file_name: {:?}", file_name);
            fixed_args.push(
                file_re
                    .replace_all(str, format!("file_name={}", file_name).as_str())
                    .to_string(),
            );
            // continue;
            // debug!("str after file re {:?}", str);
        } else {
            fixed_args.push(
                re.replace_all(str, format!("={}", rand).as_str())
                    .to_string(),
            ); // TODO: this may need an iter to loop through all=* args to give different random numbers for different args
               // println!("{}", str);
               // debug!("str after num re {:?}", str);
        }
    }
    // debug!("command args after mod is: {:?}", fixed_args.clone());

    if let Ok(mut child) = std::process::Command::new(program.cmd.as_ref().unwrap())
        .args(fixed_args.clone())
        .spawn()
    {
        debug!("spawned the child process");
        child.wait().expect("command wasn't running");
        info!("Child has finished its execution!");
    } else {
        debug!("child command didn't start");
    }

    // std::process::Command::new(program.cmd.as_ref().unwrap())
    //     .args(fixed_args.clone())
    //     .output()
    //     .expect("error on child command");

    let finish_time = Utc::now();
    let duration = (finish_time - unfinished_log.start).num_milliseconds();

    // FIXME: this might be wrong since unfinished_log is not &mut
    unfinished_log.finish = Some(finish_time);
    unfinished_log.duration = Some(duration);
    unfinished_log.args = Some(fixed_args);

    unfinished_log.to_owned()
}

#[cfg(test)]
mod tests {
    use crate::corun::Program;
    use rand::Rng;
    use regex::Regex;
    use std::fs::File;
    use std::io::{self, Write};
    use std::{env, path::Path};

    #[test]
    fn test() {
        let a = ["1", "two", "NaN", "four", "5"];

        let mut iter = a.into_iter().filter_map(|s| s.parse::<String>().ok());
        println!("{:#?}", iter);
        println!("{:#?}", a);
        let re = Regex::new(r"file_name=\d+").unwrap();
        let mut str = "file_name=*".to_string();

        let mut p = Program {
            cmd: Some("make".to_string()),
            path: Some("PolyBenchC-4.2.1/linear-algebra/blas/gemm/".into()),
            range: Some((500, 5000)),
            args: Some(vec!["run".to_string(), "N=*".to_string()]),
        };
        let tmp = p.path.clone().unwrap();
        for i in 0..3 {
            let t = Path::new("PolyBenchC-4.2.1/linear-algebra/blas/gemm/");
            println!("{}", i);
            println!("curr dir: {:#?}", std::env::current_dir());
            if !std::env::current_dir().unwrap().ends_with(t) {
                println!("in here");
                assert!(std::env::set_current_dir(t).is_ok());
            }
        }

        println!("out");

        for str in p.args.as_mut().unwrap_or(&mut Vec::new()) {
            let re = Regex::new(r"\=\*").unwrap();
            let rand = rand::thread_rng().gen_range(p.range.unwrap().0..=p.range.unwrap().1);
            *str = re
                .replace_all(&str, format!("={}", rand).as_str())
                .to_string();
            // println!("{}", str);
        }

        println!(
            "at {:#?}, exec {:#?} with {:#?}",
            std::env::current_dir().unwrap(),
            std::env::current_exe(),
            p.args.clone()
        );

        let mut cc = Vec::new();
        for i in 0..4 {
            let mut c = std::process::Command::new(p.cmd.clone().unwrap())
                .args(p.args.clone().unwrap_or(Vec::new()))
                .spawn()
                .expect("failed to execute child");
            println!("start time: {:?}", chrono::Utc::now());
            cc.push(c);
        }

        for mut c in cc {
            c.wait().expect("TODO: panic message");
            println!("end time: {:?}", chrono::Utc::now());
        }

        // let mut c = std::process::Command::new(p.cmd.unwrap())
        //     .args(p.args.unwrap_or(Vec::new()))
        //     .spawn()
        //     .expect("failed to execute child");
        // c.wait().expect("TODO: panic message");
        println!("here");
    }
}
