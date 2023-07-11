use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
// use clap::error::ContextValue::String;
use std::string::String;

use tracing::{info, trace};

// TODO: need logging(tracing and tracing-sub) on each program start time and finish time and the order thy ran(√), continuous scheduling, randomly schedule multiple programs(i actually didn't feel the need for this)
// TODO: need to add git submodule for polybench
// FIXME: probably more helpful if we read in program paths and arguments from a file(√)
// TODO: do permutation on the program args and other programs, and then use them as program pools
// TODO: need to add back tracing for the program start time and finish time using logging in order to tell which programs are running at given time, thus can figure out the DMC for that given time

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub(crate) path: Option<PathBuf>,
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

pub fn co_run(programs: Vec<Program>) -> HashMap<Program, std::time::Duration> {
    let mut log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("log");
    let mut timer = HashMap::new();
    let mut children = Vec::new();
    let mut logger = HashMap::new();
    for program in programs {
        let child = single_run(program.clone());
        let pid = child.id();
        let start = std::time::Instant::now(); // FIXME: should this be here or right after the .wait()?
        logging(
            0,
            &timer,
            &program,
            pid,
            start,
            log_file
                .as_mut()
                .unwrap_or_else(|_| panic!("log file is not created")),
        );
        timer.insert(pid, (start, program.clone()));
        children.push(child);
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
        logger.insert(program, duration);
    }
    logger
}

#[inline(always)]
fn single_run(program: Program) -> std::process::Child {
    let Some(path) = program.path else {
        panic!("program path is not specified");
    };
    let args = program.args.unwrap_or(Vec::new());
    std::process::Command::new(path)
        .args(args)
        .spawn()
        .expect("failed to execute child")
}

#[cfg(test)]
mod tests {}
