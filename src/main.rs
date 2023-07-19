#![feature(extract_if)]

use clap::Parser;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tracing::Level;
// use tracing_subscriber::fmt::writer::MakeWriterExt;

// use tracing_subscriber::prelude::*;
mod corun;
mod logging;
mod utils;

// TODO: need to change this to a file input (√)

#[derive(Debug, Parser)]
pub struct CliInput {
    #[arg(long, short, help = "how long these programs should run for in hours")]
    pub duration: Option<f64>,
    #[arg(long, short, help = "program command")]
    pub cmd: Option<String>,
    #[arg(long, short, help = "program path")]
    pub path: Option<PathBuf>,
    #[arg(long, short, help = "program arguments")]
    pub args: Option<Vec<String>>,
    #[arg(
        long,
        short,
        help = "program parameters range(i32-i32, separated by dash))"
    )]
    pub range: Option<String>,
}

#[derive(Debug, Parser)]
pub struct FileInput {
    #[arg(
        long,
        short,
        help = "real path to the config file of the co-run programs"
    )]
    pub config_path: Option<PathBuf>,
    #[arg(long, short, help = "how long these programs should run for in hours")]
    pub duration: Option<f64>,
}

#[derive(Debug, Parser)]
enum Command {
    #[allow(non_camel_case_types)]
    #[command(
        about = "input co-run programs by hand; duration, command, real path, range of the arg(s), and argument(s)"
    )]
    line_args(CliInput),
    #[allow(non_camel_case_types)]
    #[command(
        about = "input co-run programs by config file; same order as the line-args except the duration, spaced out by whitespaces"
    )]
    file_args(FileInput),
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env("CORUN_LOG");
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;
    let cpu_info = utils::get_cpu_info();
    // let cpu_info = 4;
    let command: Command = Command::parse();
    let mut log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("log");
    match command {
        Command::line_args(opt) => {
            let program_pool = vec![corun::Program {
                cmd: opt.cmd,
                path: opt.path,
                range: Option::from(utils::get_range(opt.range)),
                args: opt.args,
            }];
            let duration = utils::get_duration(opt.duration);
            let duration_hash = corun::co_run(program_pool, duration, cpu_info, &mut log_file);
            println!("durations: {:#?}", duration_hash);
        } // FIXME: this seems to have some problem with continuous inputs

        Command::file_args(opt) => {
            let config_path = opt
                .config_path
                .unwrap_or_else(|| panic!("config file path is not specified"));
            // get all the lines from the file
            let contents = fs::read_to_string(config_path)?;
            let mut program_pool = Vec::new();
            let duration = utils::get_duration(opt.duration);

            for line in contents.lines() {
                //hope this works
                let mut iter = line.split_whitespace();

                let program = corun::Program {
                    cmd: Option::from(
                        iter.next()
                            .unwrap_or_else(|| panic!("program command is not specified"))
                            .to_string(),
                    ),
                    path: Option::from(PathBuf::from(
                        iter.next()
                            .unwrap_or_else(|| panic!("program path is not specified")),
                    )),
                    range: Option::from(utils::get_range(Some(
                        iter.next()
                            .unwrap_or_else(|| panic!("program path is not specified"))
                            .to_string(),
                    ))),
                    args: Option::from(
                        iter.collect::<Vec<&str>>()
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                    ),
                };
                program_pool.push(program);
            }
            let duration_hash = corun::co_run(program_pool, duration, cpu_info, &mut log_file);
            println!("durations: {:#?}", duration_hash);
            log_file
                .unwrap()
                .write_all(format!("durations: {:#?}", duration_hash).as_bytes())
                .unwrap();
        }
    }
    Ok(())
}
