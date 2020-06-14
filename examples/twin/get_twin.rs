
use azure_iothub_service::IoTHubService;

#[tokio::main]
async fn main() {
    let iothub_service = IoTHubService::from_private_key("some-iot-hub", "YSB2ZXJ5IHNlY3VyZSBrZXkgaXMgaW1wb3J0YW50Cg==", 3600).expect("Failed to create IoTHubService");
    let twin_manager = iothub_service.twin_manager();

    let edge_agent_twin = twin_manager.get_module_twin("some-device", "SomeModule").await.expect("Failed to retrieve edgeAgent twin");
    let device_twin = twin_manager.get_device_twin("some-device").await.expect("Failed to retrieve the device twin");

    println!("EdgeAgent connectionState: {}", edge_agent_twin.connection_state);
    println!("Device connection state: {}", device_twin.connection_state);
}