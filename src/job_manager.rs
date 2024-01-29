use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use crate::api::{StatusResponse, StatusType};

/// Job represents a command that has been requested to run by a client.
#[derive(Debug)]
struct Job {
    /// id uniquely identifies the job
    id: Uuid,

    /// status contains the current status of the job
    status: StatusResponse,

    /// command is the command that was requested. It's a String (rather than a &str) because the Job owns the content.
    command: String,

    child: unshare::Child,
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
        let mut guard = self.jobs.lock().unwrap();

        //TODO: run child process here and capture output. Don't worry about streaming the logs yet
        match crate::reexec::get_child(command.clone()) {
            Err(e) =>
            {
                log::error!("error: {}", e);
                Err("failed to run child")
            },
            Ok(child) => {
                let job = Job {
                    id,
                    //TODO: status should come from the child.
                    status: StatusResponse {
                        status: StatusType::Running as i32,
                        exit_code: 0,
                    },
                    command,
                    child,
                };
                log::info!("Created job {:?}", &job);

                guard.insert(id, job);
                Ok(id)
            }            
        }
    }

    /// Return the status of the job identified by the given UUID
    pub fn status(&self, uuid: Uuid) -> Result<StatusResponse, &str> {
        let guard = self.jobs.lock().unwrap();

        let command = guard.get(&uuid);
        match command {
            //TODO: status should come from the child. It's actually not clear how I can get that information here.
            Some(job) => {
                //TODO tidy this up
                let pid: i32 = job.child.pid();
                let x = nix::unistd::Pid::from_raw(pid);
                //TODO WNOWAIT gave me EINVAL for some reason.
                //let result = nix::sys::wait::waitpid(x, Some(nix::sys::wait::WaitPidFlag::WNOWAIT));
                let result = nix::sys::wait::waitpid(x, Some(nix::sys::wait::WaitPidFlag::WNOHANG));

                //TODO this nested match is not great. What's a better approach?
                match result {
                    Err(e) => {
                        log::error!("error getting status: {}", e);
                        Err("error getting status")
                    },
                    Ok(status) => {
                        match status {

                            //TODO lots to tidy up here, but the basic idea works
                            // test this with a longer running command
                            nix::sys::wait::WaitStatus::StillAlive => {
                                log::info!("status: still alive");
                                return Ok(crate::api::StatusResponse {
                                    status: StatusType::Running as i32,
                                    exit_code: 0,
                                })
                            },
                            // this does seem to be working.
                            nix::sys::wait::WaitStatus::Exited(p, status) => {
                                log::info!("status of {}: {}", p, status);
                            },
                            e => {
                                log::info!("other status");
                            },
                        }
                        //log::info!("status: {}", status);
                        Ok(job.status.clone())
                            
                    },
                }
            },
            None => Err("Job not found"),
        }
    }

    /// Kill the job identified by the given UUID
    pub fn kill(&self, uuid: Uuid) -> Result<(), &str> {
        let mut guard = self.jobs.lock().unwrap();

        let command = guard.get_mut(&uuid);
        match command {
            Some(mut job) => {
                job.status = crate::api::StatusResponse {
                    //TODO status should come from the child
                    status: StatusType::Stopped as i32,
                    exit_code: 0,
                };
                let result = job.child.kill();
                match result {
                    Err(e) => {
                        log::error!("error killing job: {}", e);
                        Err("Failed to kill job")
                    },
                    Ok(_) => Ok(())
                }
            }
            None => Err("Job not found"),
        }
    }
}
