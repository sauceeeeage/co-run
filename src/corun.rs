use std::path::PathBuf;

// TODO: need logging(tracing and tracing-sub) on each program start time and finish time and the order thy ran, continuous scheduling, randomly schedule multiple programs
// TODO: need to add git submodule for polybench
// FIXME: probably more helpful if we read in program paths and arguments from a file

pub struct Program {
    pub(crate) path: Option<PathBuf>,
    pub(crate) args: Option<Vec<String>>,
}

pub fn co_run(programs: Vec<Program>) -> Vec<std::time::Duration> {
    let mut children = Vec::new();
    for program in programs {
        let child = single_run(program);
        children.push(child);
    }
    let mut durations = Vec::new();
    for mut child in children {
        let start = std::time::Instant::now();
        let _ = child.wait().expect("command wasn't running");
        let duration = start.elapsed();
        durations.push(duration);
    }
    durations
}

#[inline(always)]
fn single_run(program: Program) -> std::process::Child {
    let Some(path) = program.path else {
        panic!("program path is not specified");
    };
    // let Some(args) = program.args.clone() else {
    //
    // };
    let args = program.args.unwrap_or(Vec::new());
    std::process::Command::new(path)
        .args(args)
        .spawn()
        .expect("failed to execute child")
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
}
