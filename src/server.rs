use std::pin::Pin;

use tonic::{transport::Server, Request, Response, Status};
use futures::Stream;

use api::worker_server::{Worker, WorkerServer};
use api::{Command, Empty, JobId, Log, StatusResponse};
use log::LevelFilter;

pub mod api {
    tonic::include_proto!("api");
}

#[derive(Debug, Default)]
pub struct WorkerService {}

#[tonic::async_trait]
impl Worker for WorkerService {
    async fn submit(&self, request: Request<api::Command>) -> Result<Response<JobId>, Status> {
        log::info!("Got a request: {:?}", request.get_ref());

        // let reply = hello_world::HelloReply {
        //     message: format!("Hello {}!", request.into_inner().name).into(),
        // };
        //
        // Ok(Response::new(reply))
        Result::Err(Status::unimplemented("submit is not yet implemented"))
    }

    async fn stop(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<api::Empty>, tonic::Status> {
        log::info!("stopping {:?}", request.get_ref());
        Result::Err(Status::unimplemented("stop is not yet implemented"))
    }

    async fn status(&self, request: tonic::Request<JobId>, ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        log::info!("status {:?}", request.get_ref());
        Result::Err(Status::unimplemented("status is not yet implemented"))
    }

    type GetLogsStream = Pin<Box<dyn Stream<Item = Result<api::Log, Status>> + Send + Sync + 'static>>;

    async fn get_logs(&self, request: tonic::Request<JobId>) -> Result<tonic::Response<Self::GetLogsStream>, tonic::Status> {
        log::info!("get_logs {:?}", request.get_ref());
        Result::Err(Status::unimplemented("get_logs is not yet implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Starting...");
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();

    let addr = "[::1]:50051".parse()?;
    let worker = WorkerService::default();

    Server::builder()
        .add_service(WorkerServer::new(worker))
        .serve(addr)
        .await?;

    Ok(())
}