use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use tracing::{info, trace};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Log {
    pub(crate) start: DateTime<Utc>,
    pub(crate) finish: Option<DateTime<Utc>>,
    pub(crate) duration: Option<std::time::Duration>,
    pub(crate) prog_id: u32,
    pub(crate) prog_name: String,
    pub(crate) cmd: Option<String>,
    pub(crate) args: Option<Vec<String>>,
}

pub fn logging(
    current_prog_status: usize,
    pid: u32,
    curr_duration: Option<std::time::Duration>,
    curr_prog_log: Log,
    total_log: &mut HashMap<u32, Log>,
    log_file: &mut File,
) {
    match current_prog_status {
        0 => {
            // 0 for start
            trace!(
                "program {:?}(prog_id: {:?}) started with {:?} args at {:?} time",
                curr_prog_log.prog_name,
                curr_prog_log.prog_id,
                curr_prog_log.args,
                curr_prog_log.start
            );
            log_file
                .write_all(
                    format!(
                        "program {:?}(prog_id: {:?}) started with {:?} args at {:?} time\n",
                        curr_prog_log.prog_name,
                        curr_prog_log.prog_id,
                        curr_prog_log.args,
                        curr_prog_log.start
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
        1 => {
            // 1 for finish
            trace!(
                "program {:?}(prog_id: {:?}) finished with {:?} args at {:?} time, used {:?} sec",
                curr_prog_log.prog_name,
                curr_prog_log.prog_id,
                curr_prog_log.args,
                curr_prog_log.finish.unwrap(),
                curr_duration.unwrap().as_secs()
            );
            log_file
                .write_all(
                    format!(
                        "program {:?}(prog_id: {:?}) finished with {:?} args at {:?} time, used {:?} sec\n",
                        curr_prog_log.prog_name,
                        curr_prog_log.prog_id,
                        curr_prog_log.args,
                        curr_prog_log.finish.unwrap(),
                        curr_duration.unwrap().as_secs()
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
        _ => {
            panic!("current program status unknown");
        }
    }
    total_log.insert(pid, curr_prog_log);
    info!("all programs ran: {:#?}", total_log);
}
