use std::env;

use azure_iothub_service::{IoTHubService, ModulesContent, ModulesContentBuilder};
use serde_json::json;

#[tokio::main]
async fn main() {
    let iot_hub_name =
        env::var("IOT_HUB_NAME").expect("IOT_HUB_NAME environment variable is not set");
    let private_key = env::var("IOT_HUB_PRIVATE_KEY")
        .expect("IOT_HUB_PRIVATE_KEY environment variable is not set");

    let iothub_service = IoTHubService::from_private_key(iot_hub_name, private_key, 3600)
        .expect("Failed to create IoTHubService");
    let query = iothub_service
        .build_query()
        .select("*")
        .from("devices")
        .build()
        .expect("Failed to build the query");

    let query_result = query.execute().await.expect("Failed to execute the query");
    println!(
        "{}",
        serde_json::to_string_pretty(&query_result)
            .expect("Failed to convert JSON to pretty string")
    );
}
