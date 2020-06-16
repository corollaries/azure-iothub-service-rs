use std::env;

use azure_iothub_service::IoTHubService;

#[tokio::main]
async fn main() {
    let iot_hub_name =
        env::var("IOT_HUB_NAME").expect("IOT_HUB_NAME environment variable is not set");
    let private_key = env::var("IOT_HUB_PRIVATE_KEY")
        .expect("IOT_HUB_PRIVATE_KEY environment variable is not set");
    let device_id = env::var("DEVICE_ID").expect("DEVICE_ID environment variable is not set");

    let iothub_service = IoTHubService::from_private_key(iot_hub_name, private_key, 3600)
        .expect("Failed to create IoTHubService");
    let twin_manager = iothub_service.twin_manager();

    let edge_agent_twin = twin_manager
        .get_module_twin(&device_id, "$edgeAgent")
        .await
        .expect("Failed to retrieve edgeAgent twin");
    let device_twin = twin_manager
        .get_device_twin(&device_id)
        .await
        .expect("Failed to retrieve the device twin");

    println!(
        "EdgeAgent connectionState: {}",
        edge_agent_twin.connection_state
    );
    println!("Device connection state: {}", device_twin.connection_state);
}
