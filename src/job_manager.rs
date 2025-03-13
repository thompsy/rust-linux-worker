use crate::api::{StatusResponse, StatusType};
use crate::reexec;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use unshare::Command;
use std::sync::Arc;

/// Job represents a command that has been requested to run by a client.
#[derive(Debug)]
struct Job {
    /// id uniquely identifies the job
    id: Uuid,

    /// status contains the current status of the job
    status: StatusResponse,

    /// command is the command that was requested. It's a String (rather than a &str) because the Job owns the content.
    command: String,

    //cmd: Arc<Command>,
}

/// JobManager manages the submitted jobs.
#[derive(Debug)]
pub struct JobManager {
    jobs: Mutex<HashMap<Uuid, Job>>,
}

impl JobManager {
    /// Return a new JobManager with an empty jobs map.
    pub fn new() -> JobManager {
        JobManager {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    /// Submit the given command to be executed
    pub fn submit(&self, command: String) -> Result<Uuid, &str> {
        let id = Uuid::new_v4();

        let c = reexec::get_child(&command);
        let cmd = Arc::new(c);

        let job = Job {
            id,
            status: StatusResponse {
                status: StatusType::Running as i32,
                exit_code: 0,
            },
            command,
            //cmd,
        };
        log::info!("Created job {:?}", &job);

        //TODO: run child process here and capture output. Don't worry about streaming the logs yet

        // TODO: consider adding the actual job to the struct
        let mut guard = self.jobs.lock().unwrap();
        guard.insert(id, job);

        Ok(id)
    }

    /// Return the status of the job identified by the given UUID
    pub fn status(&self, uuid: Uuid) -> Result<StatusResponse, &str> {
        let guard = self.jobs.lock().unwrap();

        let command = guard.get(&uuid);
        match command {
            Some(job) => {
                let status = job.status.clone();
                log::info!("found job {:?} with status: {:?}", uuid, status);
                Ok(status)
            }

            None => {
                log::info!("job not found {:?}", uuid);
                Err("Job not found")
            }
        }
    }

    /// Kill the job identified by the given UUID
    pub fn kill(&self, uuid: Uuid) -> Result<(), &str> {
        let mut guard = self.jobs.lock().unwrap();

        let command = guard.get_mut(&uuid);
        match command {
            Some(job) => {
                log::info!("found job {:?}", uuid);
                // TODO we actually need to kill the process here!
                job.status = crate::api::StatusResponse {
                    status: StatusType::Stopped as i32,
                    exit_code: 0,
                };
                Ok(())
            }
            None => {
                log::info!("job not found {:?}", uuid);
                Err("Job not found")
            }
        }
    }
}
