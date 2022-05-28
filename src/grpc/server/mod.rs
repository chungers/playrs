#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use super::hello::hello_service_server::{HelloService, HelloServiceServer};
use super::hello::{HelloRequest, HelloResponse};

use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MyServer {}

#[tonic::async_trait]
impl HelloService for MyServer {
    async fn hello_world(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let mut out: String = "Hello, ".to_owned();
        out.push_str(&request.into_inner().name);

        info!("response message is {:?}", out);

        let response = HelloResponse {
            message: out.to_string(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
pub async fn start(addr: &String) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?; // 0.0.0.0:5001

    info!("Starting MyServer at {:?}", addr);

    let hello_server = MyServer::default();
    Server::builder()
        .add_service(HelloServiceServer::new(hello_server))
        .serve(addr)
        .await?;

    Ok(())
}
