use chrono::{DateTime, Utc};

use std::sync::Arc;
use std::sync::Mutex;
use tracing::trace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Log {
    pub(crate) start: DateTime<Utc>,
    pub(crate) finish: Option<DateTime<Utc>>,
    pub(crate) duration: Option<i64>,
    pub(crate) prog_id: u64,
    pub(crate) prog_name: String,
    pub(crate) cmd: Option<String>,
    pub(crate) args: Option<Vec<String>>,
}

pub fn logging(current_prog_status: usize, curr_prog_log: Log, total_log: Arc<Mutex<Vec<Log>>>) {
    let mut total_log = total_log.lock().unwrap();
    // let log_file = log_file.lock().unwrap();
    // let mut log_file = log_file.as_ref().unwrap();
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
            // log_file
            //     .write_all(
            //         format!(
            //             "program {:?}(prog_id: {:?}) started with {:?} args at {:?} time\n",
            //             curr_prog_log.prog_name,
            //             curr_prog_log.prog_id,
            //             curr_prog_log.args,
            //             curr_prog_log.start
            //         )
            //         .as_bytes(),
            //     )
            //     .unwrap();
        }
        1 => {
            // 1 for finish
            trace!(
                "program {:?}(prog_id: {:?}) started at{:?}, finished with {:?} args at {:?} time, used {:?} mili sec",
                curr_prog_log.prog_name,
                curr_prog_log.prog_id,
                curr_prog_log.start,
                curr_prog_log.args,
                curr_prog_log.finish.unwrap(),
                curr_prog_log.duration
            );
            // log_file
            //     .write_all(
            //         format!(
            //             "program {:?}(prog_id: {:?}) started at{:?}, finished with {:?} args at {:?} time, used {:?} mili sec\n",
            //             curr_prog_log.prog_name,
            //             curr_prog_log.prog_id,
            //             curr_prog_log.start,
            //             curr_prog_log.args,
            //             curr_prog_log.finish.unwrap(),
            //             curr_prog_log.duration
            //         )
            //         .as_bytes(),
            //     )
            //     .unwrap();
            total_log.push(curr_prog_log);
            // info!("all programs ran: {:#?}", total_log);
        }
        _ => {
            panic!("current program status unknown");
        }
    }
}
