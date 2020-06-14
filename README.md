# azure-iothub-service

`azure-iothub-service` is an async wrapper around the REST API of Azure IoT Hub. The goal of this library is to simplify creating applications that need to use the REST API. 

> Note: This library is a personal project and work in progress. Breaking changes can happen at any time.

## Examples

### Get Module and Device Twin
```rust
let iothub_service = IoTHubService::from_private_key("some-iot-hub", "YSB2ZXJ5IHNlY3VyZSBrZXkgaXMgaW1wb3J0YW50Cg==", 3600).expect("Failed to create IoTHubService");
let twin_manager = iothub_service.twin_manager();

let edge_agent_twin = twin_manager.get_module_twin("some-device", "SomeModule").await.expect("Failed to retrieve edgeAgent twin");
let device_twin = twin_manager.get_device_twin("some-device").await.expect("Failed to retrieve the device twin");

println!("EdgeAgent connectionState: {}", edge_agent_twin.connection_state);
println!("Device connection state: {}", device_twin.connection_state);
```

### Invoke a Module Method
```rust
let iothub_service = IoTHubService::from_private_key("some-iot-hub", "YSB2ZXJ5IHNlY3VyZSBrZXkgaXMgaW1wb3J0YW50Cg==", 3600).expect("Failed to create IoTHubService");
let module_method = iothub_service.create_module_method("some-device", "$edgeAgent", "ping", 10, 20);
let response = module_method.invoke(json!({})).await.expect("Failed to invoke ping on edgeAgent");

println!("Invoking 'ping' on edgeAgent returned with: {}", response.status);
```

### Apply Modules Configuration
```rust
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
```

## Implemented features
### Configuration
- [X] Apply `modules` configuration
- [ ] Apply `module` configuration
- [ ] Apply `device` configuration
- [ ] Create or update
- [ ] Delete
- [ ] Get 
- [ ] Get Configurations

### Direct methods
- [X] Invoke a device method
- [X] Invoke a module method

### Fault injection
- [ ] Get
- [ ] Set

### Http Runtime
- [ ] Abandon Feedback Notification
- [ ] Complete Feedback Notification
- [ ] Receive Feedback Notification

### Job Client
- [ ] Cancel Import Export Job
- [ ] Cancel Job 
- [ ] Create Import Export Job 
- [ ] Create Job
- [ ] Get Import Export Job
- [ ] Get Import Export Jobs 
- [ ] Get Job 
- [ ] Query Jobs

### Registry Manager
- [ ] Bulk Device CRUD
- [ ] Create Or Update Device
- [ ] Create Or Update Module
- [ ] Delete Device
- [ ] Delete Module
- [ ] Get Device 
- [ ] Get Device Statistics
- [X] Get Devices (via Query IoT Hub)
- [X] Get Module (via Query IoT Hub)
- [X] Get Modules On Device (via Query IoT Hub)
- [ ] Get Service Statistics
- [ ] Purge Command Queue
- [X] Query IoT Hub

### Twins
- [X] Get device twin
- [X] Get module twin
- [X] Replace device twin
- [X] Replace module twin
- [X] Update device twin
- [X] Update module twin

