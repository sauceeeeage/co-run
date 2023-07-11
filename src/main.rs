use clap::Parser;
use std::fs;
use std::path::PathBuf;

// use tracing_subscriber::prelude::*;
mod corun;

// TODO: need to change this to a file input (√)

#[derive(Debug, Parser)]
pub struct CliInput {
    #[arg(long, short, help = "program path")]
    pub path: Option<PathBuf>,
    #[arg(long, short, help = "program arguments")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Parser)]
pub struct FileInput {
    #[arg(long, short, help = "config file real path")]
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Parser)]
enum Command {
    #[allow(non_camel_case_types)]
    #[command(about = "co-run programs real paths and arguments")]
    line_args(CliInput),
    #[allow(non_camel_case_types)]
    #[command(about = "real path to the config file of the co-run programs")]
    file_args(FileInput),
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env("CORUN_LOG");
    let command: Command = Command::parse();
    match command {
        Command::line_args(opt) => {
            let programs = vec![corun::Program {
                path: opt.path,
                args: opt.args,
            }];
            let durations = corun::co_run(programs);
            println!("durations: {:#?}", durations);
        }
        Command::file_args(opt) => {
            let config_path = opt
                .config_path
                .unwrap_or_else(|| panic!("config file path is not specified"));
            // get all the lines from the file
            let contents = fs::read_to_string(config_path)?;
            let mut programs = Vec::new();
            for line in contents.lines() {
                //hope this works
                let mut program = corun::Program {
                    path: None,
                    args: None,
                };
                let mut iter = line.split_whitespace();
                let path = iter
                    .next()
                    .unwrap_or_else(|| panic!("program path is not specified"));
                let args = iter.collect::<Vec<&str>>();
                program.path = Some(PathBuf::from(path));
                program.args = Some(args.iter().map(|s| s.to_string()).collect::<Vec<String>>());
                programs.push(program);
            }
            let durations = corun::co_run(programs);
            println!("durations: {:#?}", durations);
        }
    }
    Ok(())
}
