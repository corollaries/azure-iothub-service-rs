use std::env;

use azure_iothub_service::{IoTHubService, ModulesContent, ModulesContentBuilder};
use serde_json::json;

#[tokio::main]
async fn main() {
    let iot_hub_name =
        env::var("IOT_HUB_NAME").expect("IOT_HUB_NAME environment variable is not set");
    let private_key = env::var("IOT_HUB_PRIVATE_KEY")
        .expect("IOT_HUB_PRIVATE_KEY environment variable is not set");
    let device_id = env::var("DEVICE_ID").expect("DEVICE_ID environment variable is not set");

    let iothub_service = IoTHubService::from_private_key(iot_hub_name, private_key, 3600)
        .expect("Failed to create IoTHubService");
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
        .build()
        .expect("Failed to create configuration");

    iothub_service
        .apply_modules_configuration(device_id, &modules_content)
        .await
        .expect("Failed to apply configuration");
}
