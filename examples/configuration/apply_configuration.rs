use azure_iothub_service::{IoTHubService, ModulesContent, ModulesContentBuilder};
use serde_json::json;

#[tokio::main]
async fn main() {
    let iothub_service = IoTHubService::from_private_key("some-iot-hub", "YSB2ZXJ5IHNlY3VyZSBrZXkgaXMgaW1wb3J0YW50Cg==", 3600).expect("Failed to create IoTHubService");
    let modules_content = ModulesContentBuilder::new()
        .minimum_docker_version("v1.25")
        .edge_agent_image("mcr.microsoft.com/azureiotedge-agent:1.0")
        .edge_agent_create_options(json!({
            "HostConfig": {
                "PortBindings": {
                    "443/tcp": [{"HostPort": "443"}],
                    "5671/tcp": [{"HostPort": "5671"}],
                    "8883/tcp": [{"HostPort": "8883"}]
                }
            }
        }))
        .edge_hub_image("mcr.microsoft.com/azureiotedge-hub:1.0")
        .time_to_live_secs(9600)
        .build().expect("Failed to create configuration");
    
    iothub_service.apply_modules_configuration("some-device", &modules_content).await.expect("Failed to apply configuration");
}

