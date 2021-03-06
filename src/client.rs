use api::worker_client::WorkerClient;
use clap::{AppSettings, Clap};
use log::LevelFilter;
use uuid::Uuid;

/// api is the namespace for the generated GRPC code.
pub mod api {
    tonic::include_proto!("api");
}

/// rust-linux-worker-client is a client to sent commands to the rust-linux-worker-server for execution.
#[derive(Clap)]
#[clap(version = "1.0", author = "Andrew Thompson <code@downthewire.co.uk>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Submit(Submit),
    Status(Status),
    Logs(Logs),
    Kill(Kill),
}

/// Submit a job to the server
#[derive(Clap)]
struct Submit {
    /// Command to submit
    command: String,
}

/// Query the status of a job
#[derive(Clap)]
struct Status {
    /// The job_id to fetch the status for
    job_id: String,
}

/// Fetch the logs for a job
#[derive(Clap)]
struct Logs {
    /// JobId to fetch the logs for
    job_id: String,
}

/// Kill a job
#[derive(Clap)]
struct Kill {
    /// JobId to kill
    job_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let opts: Opts = Opts::parse();

    let mut client = WorkerClient::connect("http://0.0.0.0:50051".to_string()).await?;

    match opts.subcmd {
        SubCommand::Submit(s) => {
            log::info!("Submitting command {}", s.command);
            let request = tonic::Request::new(api::Command { command: s.command });
            let response = client.submit(request).await?;

            log::info!("response {:?}", response)
        }
        SubCommand::Status(s) => {
            log::info!("Getting status for job_id {}", &s.job_id);

            let request = tonic::Request::new(api::JobId { id: s.job_id.to_owned() });
            let response = client.status(request).await?;

            log::info!("response {:?}", response)
        }
        SubCommand::Logs(s) => {
            let uuid = parse_uuid_or_abort(s.job_id);
            log::info!("Fetching logs for job_id {}", uuid)
        }
        SubCommand::Kill(s) => {
            log::info!("Killing job_id {}", &s.job_id);

            let request = tonic::Request::new(api::JobId { id: s.job_id.to_owned() });
            let response = client.stop(request).await?;

            log::info!("response {:?}", response)
        }
    };

    Ok(())
}

/// Parse the given UUID or abort the process if parsing fails.
fn parse_uuid_or_abort(input: String) -> Uuid {
    match Uuid::parse_str(input.as_str()) {
        Err(e) => {
            log::error!("Error parsing UUID: {}", e);
            std::process::exit(1)
        }
        Ok(uuid) => uuid,
    }
}
