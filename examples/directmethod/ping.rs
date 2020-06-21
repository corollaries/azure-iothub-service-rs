use std::env;

use azure_iothub_service::IoTHubService;
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
    let module_method =
        iothub_service.create_module_method(device_id, "$edgeAgent", "ping", 10, 20);
    let response = module_method
        .invoke::<serde_json::Value>(json!({}))
        .await
        .expect("Failed to invoke ping on edgeAgent");

    println!(
        "Invoking 'ping' on edgeAgent returned with: {}",
        response.status
    );
}
