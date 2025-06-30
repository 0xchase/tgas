use tonic::transport::Channel;

pub mod ipv6kit {
    tonic::include_proto!("ipv6kit");
}

use ipv6kit::ipv6_kit_service_client::Ipv6KitServiceClient;
use ipv6kit::{CommandRequest, CommandResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing gRPC connection...");
    
    // Try to connect to the server
    let client = Ipv6KitServiceClient::connect("http://127.0.0.1:50051").await?;
    println!("Successfully connected to server!");
    
    // Create a simple test request
    let request = CommandRequest {
        command: "generate".to_string(),
        args: vec!["--count".to_string(), "2".to_string()],
    };
    
    println!("Sending request: {:?}", request);
    
    // Send the request
    let response = client.execute_command(request).await?;
    let response = response.into_inner();
    
    println!("Received response:");
    println!("  Success: {}", response.success);
    println!("  Output: {}", response.output);
    println!("  Error: {}", response.error);
    println!("  Exit code: {}", response.exit_code);
    
    Ok(())
} 