use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
// use clap::error::ContextValue::String;
use std::string::String;
use std::thread::{self};
use std::time;

use rand::Rng;
use regex::Regex;
use tracing::{info, trace};

// TODO: need logging(tracing and tracing-sub) on each program start time and finish time and the order thy ran(√), continuous scheduling, randomly schedule multiple programs(i actually didn't feel the need for this)
// TODO: need to add git submodule for polybench(√)
// FIXME: probably more helpful if we read in program paths and arguments from a file(√)
// TODO: do permutation on the program args and other programs, and then use them as program pools
// TODO: need to add back tracing for the program start time and finish time using logging in order to tell which programs are running at given time, thus can figure out the DMC for that given time(√)
// TODO: do GEMM(interval between 500-5000(rand num), run for 5 hr, night time(est)) polybench and get DMC(on cycle2.cs machine)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub(crate) cmd: Option<String>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) range: Option<(i32, i32)>,
    pub(crate) args: Option<Vec<String>>,
}

fn logging(
    current_prog_status: usize,
    timer: &HashMap<u32, (std::time::Instant, Program)>,
    curr_prog: &Program,
    curr_pid: u32,
    start: std::time::Instant,
    log_file: &mut File,
) {
    match current_prog_status {
        0 => {
            // 0 for start
            trace!(
                "program {:?}(pid: {:?}) started with {:?} args at {:?} time",
                curr_prog.path,
                curr_pid,
                curr_prog.args,
                start
            );
        }
        1 => {
            // 1 for finish
            trace!(
                "program {:?}(pid: {:?}) finished with {:?} args at {:?} time",
                curr_prog.path,
                curr_pid,
                curr_prog.args,
                start
            );
        }
        _ => {
            panic!("current program status is not 0 or 1");
        }
    }
    info!(
        "current running programs' starting time, names and args: {:#?}",
        timer.values()
    );
    log_file
        .write_all(
            format!(
                "current running programs' starting time, names and args: {:#?}",
                timer.values()
            )
            .as_bytes(),
        )
        .expect("unexpected error");
}

pub fn co_run(
    programs: Vec<Program>,
    total_dur: std::time::Duration,
    cpu_cnt: usize,
) -> HashMap<Program, std::time::Duration> {
    let mut log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("log");
    let mut timer = HashMap::new();
    let mut children = Vec::new();
    let mut logger = HashMap::new();
    let timer_start = std::time::Instant::now();

    while (timer_start.elapsed()) < total_dur {
        if timer.len() < cpu_cnt {
            // each thread take on program, if there's less programs than the thread num, launch one program
            let current_prog = programs[rand::thread_rng().gen_range(0..=programs.len())].clone();
            let child = single_run(&current_prog);
            let pid = child.id();
            let start = std::time::Instant::now();
            logging(
                0,
                &timer,
                &current_prog,
                pid,
                start,
                log_file
                    .as_mut()
                    .unwrap_or_else(|_| panic!("log file is not created")),
            );
            timer.insert(pid, (start, current_prog));
            children.push(child);
        } else {
            // if there's more programs than the thread num, wait for one program to finish and launch another one

            let sleep_dur = time::Duration::from_secs(2);

            thread::sleep(sleep_dur); // FIXME: having some borrowing issues below, using this for now

            // let mut child = children
            //     .iter_mut()
            //     .find(|&c| matches!(c.try_wait(), Ok(Some(_))))
            //     .unwrap_or_else(|| panic!("no child is available"));
            // let pid = child.id();
            // let (start, program) = timer.remove(&pid).unwrap_or_else(|| {
            //     panic!("program with pid: {:?} is not in the timer", child.id())
            // });
            // let duration = start.elapsed();
            // logging(
            //     1,
            //     &timer,
            //     &program,
            //     pid,
            //     start,
            //     log_file
            //         .as_mut()
            //         .unwrap_or_else(|_| panic!("log file is not created")),
            // );
            // logger.insert(program, duration); // TODO: add start and end time to the logger, and print it into the file
        }
    }

    for mut child in children {
        let _ = child.wait().expect("command wasn't running");
        let pid = child.id();
        let (start, program) = timer
            .remove(&pid)
            .unwrap_or_else(|| panic!("program with pid: {:?} is not in the timer", child.id()));
        let duration = start.elapsed();
        logging(
            1,
            &timer,
            &program,
            pid,
            start,
            log_file
                .as_mut()
                .unwrap_or_else(|_| panic!("log file is not created")),
        );
        logger.insert(program, duration); // TODO: add start and end time to the logger, and print it into the file
    }
    logger
}

#[inline(always)]
fn single_run(program: &Program) -> std::process::Child {
    let program = program.clone();
    let Some(path) = program.path else {
        panic!("program path is not specified");
    };
    assert!(std::env::set_current_dir(path).is_ok());
    let mut fixed_args = Vec::new();
    for str in program.args.unwrap_or(Vec::new()) {
        let re = Regex::new(r"\=\*").unwrap();
        let rand =
            rand::thread_rng().gen_range(program.range.unwrap().0..=program.range.unwrap().1);
        fixed_args.push(
            re.replace_all(&str, format!("={}", rand).as_str())
                .to_string(),
        );
        // println!("{}", str);
    }
    std::process::Command::new(program.cmd.unwrap())
        .args(fixed_args)
        .spawn()
        .expect("failed to execute child")
}

#[cfg(test)]
mod tests {
    use crate::corun::Program;
    use rand::Rng;
    use regex::Regex;

    #[test]
    fn test() {
        let mut p = Program {
            cmd: Some("make".to_string()),
            path: Some("PolyBenchC-4.2.1/linear-algebra/blas/gemm/".into()),
            range: Some((5, 50)),
            args: Some(vec!["run".to_string(), "N=*".to_string()]),
        };
        assert!(std::env::set_current_dir(p.path.unwrap()).is_ok());

        for str in p.args.as_mut().unwrap_or(&mut Vec::new()) {
            let re = Regex::new(r"\=\*").unwrap();
            let rand = rand::thread_rng().gen_range(p.range.unwrap().0..=p.range.unwrap().1);
            *str = re
                .replace_all(&str, format!("={}", rand).as_str())
                .to_string();
            // println!("{}", str);
        }

        let mut c = std::process::Command::new(p.cmd.unwrap())
            .args(p.args.unwrap_or(Vec::new()))
            .spawn()
            .expect("failed to execute child");
        c.wait().expect("TODO: panic message");
    }
}
