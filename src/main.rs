use clap::Parser;
use std::path::PathBuf;
mod corun;

// TODO: need to change this to a file input

#[derive(Debug, Parser)]
pub struct CliOpt {
    #[arg(long, short, help = "program path")]
    pub path: Option<PathBuf>,
    #[arg(long, short, help = "program arguments")]
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Parser)]
enum Command {
    #[allow(non_camel_case_types)]
    #[command(about = "co-run programs real paths and arguments")]
    corun(CliOpt),
}

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env("CORUN_LOG");
    let command: Command = Command::parse();
    match command {
        Command::corun(opt) => {
            let programs = vec![corun::Program {
                path: opt.path,
                args: opt.args,
            }];
            let durations = corun::co_run(programs);
            println!("durations: {:?}", durations);
        }
    }
    Ok(())
}
