use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Mutex;

use api::worker_server::{Worker, WorkerServer};
use api::{JobId, StatusResponse};
use futures::Stream;
use log::LevelFilter;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod api {
    tonic::include_proto!("api");
}

#[derive(Debug)]
pub struct WorkerService {
    my_worker: MyWorker,
}

#[tonic::async_trait]
impl Worker for WorkerService {
    async fn submit(&self, request: Request<api::Command>) -> Result<Response<JobId>, Status> {
        log::info!("Got a request: {:?}", request.get_ref());

        let result = self.my_worker.submit(request.get_ref().command.to_string());

        match result {
            Ok(uuid) => Ok(Response::new(api::JobId { id: uuid.to_string() })),
            Err(e) => Err(Status::new(tonic::Code::Internal, e)),
        }
    }

    async fn stop(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<api::Empty>, tonic::Status> {
        log::info!("stopping {:?}", request.get_ref());

        let parsed = Uuid::parse_str(&request.get_ref().id);
        if parsed.is_err() {
            return Err(Status::invalid_argument("invalid UUID"));
        }
        let result = self.my_worker.kill(parsed.unwrap());
        match result {
            Ok(_) => Ok(Response::new(api::Empty {})),
            Err(e) => Result::Err(Status::new(tonic::Code::Internal, e)),
        }
    }

    async fn status(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        log::info!("status {:?}", request.get_ref());

        let parsed = Uuid::parse_str(&request.get_ref().id);
        if parsed.is_err() {
            return Err(Status::invalid_argument("invalid UUID"));
        }
        let result = self.my_worker.status(parsed.unwrap());
        match result {
            Ok(_) => Ok(Response::new(api::StatusResponse { status: 1, exit_code: 0 })),
            Err(e) => Result::Err(Status::new(tonic::Code::Internal, e)),
        }
    }

    type GetLogsStream = Pin<Box<dyn Stream<Item = Result<api::Log, Status>> + Send + Sync + 'static>>;

    async fn get_logs(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<Self::GetLogsStream>, tonic::Status> {
        log::info!("get_logs {:?}", request.get_ref());
        Result::Err(Status::unimplemented("get_logs is not yet implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    log::info!("Starting...");

    // for some reason 127.0.0.1 didn't work here
    let addr = "0.0.0.0:50051".parse()?;
    let worker = WorkerService { my_worker: MyWorker::new() };

    Server::builder().add_service(WorkerServer::new(worker)).serve(addr).await?;

    Ok(())
}

//TODO: rename this to something better. Or maybe rename the other Worker.
#[derive(Debug)]
struct MyWorker {
    //TODO: this should be a sensible struct rather than a string in due course
    jobs: Mutex<HashMap<Uuid, String>>,
}

impl MyWorker {
    fn new() -> MyWorker {
        MyWorker {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    fn submit(&self, command: String) -> Result<Uuid, &str> {
        let id = Uuid::new_v4();
        let mut guard = self.jobs.lock().unwrap();

        guard.insert(id, command.clone());
        log::info!("Created Uuid {} for command {}", id, command);

        Ok(id)
    }

    fn status(&self, uuid: Uuid) -> Result<String, &str> {
        let guard = self.jobs.lock().unwrap();

        let command = guard.get(&uuid);
        match command {
            Some(s) => Ok(s.to_string()),
            None => Err("Job not found"),
        }
    }

    fn kill(&self, uuid: Uuid) -> Result<(), &str> {
        let guard = self.jobs.lock().unwrap();

        let command = guard.get(&uuid);
        match command {
            Some(_) => Ok(()),
            None => Err("Job not found"),
        }
    }
}
