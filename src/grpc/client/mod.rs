#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use super::hello::hello_service_client::HelloServiceClient;
use super::hello::HelloRequest;

use tonic::transport::Endpoint;
use tonic::Request;

#[tokio::main]
pub async fn call(port: &String, name: &String) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::from_shared(port.to_string())?;
    trace!("Connecting to {:?}", endpoint);

    let mut client = HelloServiceClient::connect(endpoint).await?;
    let request = Request::new(HelloRequest {
        name: name.to_string(),
    });

    let response = client.hello_world(request).await?;
    trace!("response: {:?}", response);

    println!("{}", response.into_inner().message);

    Ok(())
}
