use api::worker_server::{Worker, WorkerServer};
use api::{JobId, StatusResponse};
use clap::{AppSettings, Clap};
use futures::Stream;
use log::LevelFilter;
use std::io::prelude::*;
use std::pin::Pin;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use unshare::PipeReader;
use uuid::Uuid;

mod job_manager;
mod reexec;

/// rust-linux-worker-server runs arbitrary Linux commands in a containerised environment.
#[derive(Clap)]
#[clap(version = "1.0", author = "Andrew Thompson <code@downthewire.co.uk>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    ExecTest(ExecTest),
    Serve(Serve),
}

/// Test the execution of the given command in a containerised environment
#[derive(Clap)]
struct ExecTest {
    /// Command to execute
    command: String,
}

/// Server requests
#[derive(Clap)]
struct Serve {}

/// api is the namespace for the GRPC generated code.
pub mod api {
    tonic::include_proto!("api");
}

/// WorkerService handles the GRPC requests.
#[derive(Debug)]
pub struct WorkerService {
    /// JobManager manages the actual jobs submitted by the client.
    job_manager: job_manager::JobManager,
}

#[tonic::async_trait]
impl Worker for WorkerService {
    /// Submit a request to run the given command and return the UUID of the resulting job
    async fn submit(&self, request: Request<api::Command>) -> Result<Response<JobId>, Status> {
        log::info!("Got a request: {:?}", request.get_ref());

        let result = self.job_manager.submit(request.get_ref().command.to_string());

        match result {
            Ok(uuid) => Ok(Response::new(api::JobId { id: uuid.to_string() })),
            Err(e) => Err(Status::new(tonic::Code::Internal, e)),
        }
    }

    /// Stop the job identified by the given UUID
    async fn stop(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<api::Empty>, tonic::Status> {
        log::info!("stopping {:?}", request.get_ref());

        let parsed = Uuid::parse_str(&request.get_ref().id);
        if parsed.is_err() {
            //TODO: we should probably have error definitions shared between client and server
            return Err(Status::invalid_argument("invalid UUID"));
        }
        let result = self.job_manager.kill(parsed.unwrap());
        match result {
            Ok(_) => Ok(Response::new(api::Empty {})),
            Err(e) => Result::Err(Status::new(tonic::Code::Internal, e)),
        }
    }

    /// Query the status of the job identified by the given UUID
    async fn status(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        log::info!("status {:?}", request.get_ref());

        let parsed = Uuid::parse_str(&request.get_ref().id);
        if parsed.is_err() {
            return Err(Status::invalid_argument("invalid UUID"));
        }
        let result = self.job_manager.status(parsed.unwrap());
        match result {
            Ok(s) => Ok(Response::new(s)),
            Err(e) => Result::Err(Status::new(tonic::Code::Internal, e)),
        }
    }

    type GetLogsStream = Pin<Box<dyn Stream<Item = Result<api::Log, Status>> + Send + Sync + 'static>>;

    /// Stream the logs from the job identified by the given UUID
    async fn get_logs(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<Self::GetLogsStream>, tonic::Status> {
        log::info!("get_logs {:?}", request.get_ref());
        Result::Err(Status::unimplemented("get_logs is not yet implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        // This custom formatter allows us to include the PID in the logs for easier debugging.
        .format(|buf, record| {
            let ts = buf.timestamp();
            let p = std::process::id();
            writeln!(
                buf,
                "[{} {} {} {}] {}",
                ts,
                record.level(),
                p,
                record.module_path().unwrap(),
                record.args()
            )
        })
        .init();

    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::ExecTest(e) => {
            log::info!("re-exec test {:?}", e.command);

            //TODO here we want to create the child process, containerise it and run the command
            let mut command = reexec::get_child(&e.command);

            let mut child = command.spawn().unwrap();

            let status = child.wait();
            if status.is_err() {
                log::info!("error waiting for child process: {}", status.as_ref().unwrap_err());
            }

            let pipe: &mut PipeReader = child.stdout.as_mut().unwrap();
            let mut buffer = String::new();
            let _output = pipe.read_to_string(&mut buffer);

            log::info!("output: {}", buffer);
            log::info!("child exited with status {}", status.unwrap());
        }
        SubCommand::Serve(_) => {
            log::info!("Serving...");

            // for some reason 127.0.0.1 didn't work here
            let addr = "0.0.0.0:50051".parse()?;
            let worker = WorkerService {
                job_manager: job_manager::JobManager::new(),
            };

            Server::builder().add_service(WorkerServer::new(worker)).serve(addr).await?;
        }
    }

    Ok(())
}
