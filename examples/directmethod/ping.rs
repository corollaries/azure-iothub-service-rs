
use azure_iothub_service::IoTHubService;
use serde_json::json;


#[tokio::main]
async fn main() {
    let iothub_service = IoTHubService::from_private_key("some-iot-hub", "YSB2ZXJ5IHNlY3VyZSBrZXkgaXMgaW1wb3J0YW50Cg==", 3600).expect("Failed to create IoTHubService");
    let module_method = iothub_service.create_module_method("some-device", "$edgeAgent", "ping", 10, 20);
    let response = module_method.invoke(json!({})).await.expect("Failed to invoke ping on edgeAgent");

    println!("Invoking 'ping' on edgeAgent returned with: {}", response.status);
}