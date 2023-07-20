use std::collections::HashMap;
use std::fs::File;
// use std::io::Write;
use std::path::PathBuf;
// use clap::error::ContextValue::String;
use std::string::String;
// use std::sync::Mutex;
use std::thread::{self};
use std::time;

// use anyhow::Error;
use chrono::Utc;
use rand::Rng;
use regex::Regex;
use tracing::{debug, trace, warn};

use crate::logging::{logging, Log};
use crate::utils::delete_bin;

// TODO: need logging(tracing and tracing-sub) on each program start time and finish time and the order thy ran(√), continuous scheduling, randomly schedule multiple programs(i actually didn't feel the need for this)
// TODO: need to add git submodule for polybench(√)
// FIXME: probably more helpful if we read in program paths and arguments from a file(√)
// TODO: do permutation on the program args and other programs, and then use them as program pools
// TODO: need to add back tracing for the program start time and finish time using logging in order to tell which programs are running at given time, thus can figure out the DMC for that given time(√)
// TODO: do GEMM(interval between 500-5000(rand num), run for 5 hr, night time(est)) polybench and get DMC(on cycle2.cs machine)
// TODO: make tmpdir for each compliation, and delete them after the program finishes to make sure each concurrent programs are running the correct program(use tmpfile crate, may also need to add more cli args to the Makefile)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub(crate) cmd: Option<String>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) range: Option<(i32, i32)>,
    pub(crate) args: Option<Vec<String>>,
}

pub fn co_run(
    programs: Vec<Program>,
    total_dur: time::Duration,
    cpu_cnt: usize,
    log_file: &mut Result<File, std::io::Error>,
) -> HashMap<u32, Log> {
    let mut timer = HashMap::new();
    let mut children = Vec::new();
    // let mut logger = HashMap::new();
    let mut human_readable = HashMap::new();
    let timer_start = time::Instant::now();
    let mut prog_counter = 0;

    while (timer_start.elapsed()) < total_dur {
        if timer.len() < cpu_cnt {
            warn!("cpu is not fully utilized, launching more programs");
            // each thread take on program, if there's less programs than the thread num, launch one program
            let mut current_prog =
                programs[rand::thread_rng().gen_range(0..programs.len())].clone();
            let child = single_run(&mut current_prog, prog_counter);
            let pid = child.id();
            let start = time::Instant::now();
            let human_start = Utc::now();
            timer.insert(pid, start);
            let pn_re = Regex::new(r"\/[a-zA-Z]+\/$").unwrap();
            let prog_name = pn_re
                .captures(current_prog.path.as_ref().unwrap().to_str().unwrap())
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .replace('/', "")
                .to_string();
            let log = Log {
                start: human_start,
                finish: None,
                duration: None,
                prog_id: prog_counter,
                prog_name,
                cmd: current_prog.cmd.clone(),
                args: current_prog.args.clone(),
            };
            prog_counter += 1;
            logging(
                0,
                pid,
                None,
                log,
                &mut human_readable,
                log_file.as_mut().unwrap(),
            );

            children.push(child);
            // assert!(std::env::set_current_dir(r"~/").is_ok());
        } else {
            // if there's more programs than the thread num, wait for one program to finish and launch another one
            warn!("cpu is fully utilized, waiting for a program to finish");
            let sleep_dur = time::Duration::from_secs(2);
            thread::sleep(sleep_dur); // FIXME: having some issues below, using this for now
            let mut pid: u32 = Default::default();
            let mut start: time::Instant = time::Instant::now();
            let _ = children.iter_mut().find_map(|c| {
                matches!(c.try_wait(), Ok(Some(_))).then(|| {
                    pid = c.id();
                    // debug!("try waiting");
                    // debug!("pid: {:?}", pid);
                    // debug!("timer: {:#?}", timer);
                    start = timer.remove(&c.id()).unwrap_or_else(|| {
                        debug!("timer remove failed");
                        debug!("timer: {:#?}", timer);
                        debug!("pid: {:?}", c.id());
                        debug!("human_readable: {:#?}", human_readable);
                        // debug!("logger: {:#?}", logger);
                        panic!("program with pid: {:?} is not in the timer", c.id())
                    });
                })
            });
            // debug!("before extract if");
            // for child in children.iter() {
            //     debug!("child: {:?}", child.id());
            // }
            // debug!("children: {:#?}", children);
            let _ = children
                .extract_if(|child| child.id() == pid)
                .collect::<Vec<_>>();

            // debug!("pid: {:?}", pid);
            // debug!("after extract if");
            // for child in children.iter() {
            //     debug!("child: {:?}", child.id());
            // }
            // debug!("children: {:#?}", children);

            // let pid = child.id();
            // let start = timer.remove(&pid).unwrap_or_else(|| {
            //     panic!("program with pid: {:?} is not in the timer", child.id())
            // });
            let duration = start.elapsed();
            let human_end = Utc::now();
            let log = Log {
                start: human_readable.get(&pid).unwrap().start,
                finish: Some(human_end),
                duration: Some(duration),
                prog_id: human_readable.get(&pid).unwrap().prog_id,
                prog_name: human_readable.get(&pid).unwrap().prog_name.clone(),
                cmd: human_readable.get(&pid).unwrap().cmd.clone(),
                args: human_readable.get(&pid).unwrap().args.clone(),
            };
            delete_bin(log.prog_id.clone().to_string().into());
            logging(
                1,
                pid,
                Some(duration),
                log,
                &mut human_readable,
                log_file.as_mut().unwrap(),
            );
            trace!("program {:?} finished", pid);
            trace!("program {:?} ran for {:?}", pid, duration);
            trace!("inserting program {:?} into logger", prog_counter);
            // logger.insert(prog_counter, duration); // TODO: add start and end time to the logger, and print it into the file
        }

        // for child in children.iter_mut() {
        //     // FIXME: feels like this needs to be launched in another thread, and use non-blocking structure to wait for the child to finish
        //     warn!("waiting for child to finish");
        //     child.wait().expect("command wasn't running");

        //     let pid = child.id();
        //     let start = timer.remove(&pid).unwrap_or_else(|| {
        //         panic!("program with pid: {:?} is not in the timer", child.id())
        //     });
        //     let duration = start.elapsed();
        //     let human_end = Utc::now();
        //     let log = Log {
        //         start: human_readable.get(&pid).unwrap().start,
        //         finish: Some(human_end),
        //         prog_id: human_readable.get(&pid).unwrap().prog_id,
        //         prog_name: human_readable.get(&pid).unwrap().prog_name.clone(),
        //         cmd: human_readable.get(&pid).unwrap().cmd.clone(),
        //         args: human_readable.get(&pid).unwrap().args.clone(),
        //     };
        //     logging(
        //         1,
        //         pid,
        //         Some(duration),
        //         log,
        //         &mut human_readable,
        //         log_file.as_mut().unwrap(),
        //     );
        //     logger.insert(prog_counter, duration); // TODO: add start and end time to the logger, and print it into the file
        // }
    }
    // logger
    human_readable
}

#[inline(always)]
fn single_run(program: &mut Program, file_name: u32) -> std::process::Child {
    // let mut program = program.clone();
    // let Some(path) = program.path else {
    //     panic!("program path is not specified");
    // };
    // println!("{:?}", program.path);
    let prog_path = program.path.clone();
    if !std::env::current_dir()
        .unwrap()
        .ends_with(program.path.as_ref().unwrap())
    {
        assert!(std::env::set_current_dir(prog_path.unwrap()).is_ok());
    }
    // let mut fixed_args = Vec::new();
    for str in program.args.as_mut().unwrap_or(&mut Vec::new()) {
        let re = Regex::new(r"\=\*").unwrap();
        let file_re = Regex::new(r"file_name\=\*").unwrap();
        let rand =
            rand::thread_rng().gen_range(program.range.unwrap().0..=program.range.unwrap().1);
        if file_re.is_match(str) {
            // debug!("file_name is matched");
            // debug!("file_name: {:?}", file_name);
            *str = file_re
                .replace_all(str, format!("file_name={}", file_name).as_str())
                .to_string();
            continue;
        } else {
            *str = re
                .replace_all(str, format!("={}", rand).as_str())
                .to_string(); // TODO: this may need an iter to loop through all=* args to give different random numbers for different args
                              // println!("{}", str);
        }
    }

    std::process::Command::new(program.cmd.as_ref().unwrap())
        .args(program.args.as_ref().unwrap())
        .spawn()
        .expect("failed to execute child")
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
        // println!("{}", env::consts::OS);
        // if cfg!(target_os = "macos") {
        //     println!("macos");
        // }
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
