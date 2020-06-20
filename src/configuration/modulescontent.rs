use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize};
use serde_json::json;
use std::collections::HashMap;

use crate::error::{BuilderError, BuilderErrorType};

/// The schema version of the modulescontent
const SCHEMA_VERSION: &str = "1.0";

/// The runtime type for the containers
const RUNTIME_TYPE: &str = "docker";

/// The status of a module, either Running or Stopped
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Status {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
}

/// The restart policy of a module
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RestartPolicy {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "on-failure")]
    OnFailure,
    #[serde(rename = "on-unhealthy")]
    OnUnhealthy,
    #[serde(rename = "always")]
    Always,
}

/// The image pull policy of a module
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ImagePullPolicy {
    #[serde(rename = "on-create")]
    OnCreate,
    #[serde(rename = "never")]
    Never,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnvironmentVariable {
    value: String,
}

/// EdgeModule is an abstraction for the configuration of a custom module for IoT Edge
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeModule {
    #[serde(skip)]
    pub module_id: String,
    pub version: String,
    #[serde(rename = "type")]
    pub module_type: String,
    pub status: Status,
    pub restart_policy: RestartPolicy,
    pub image_pull_policy: Option<ImagePullPolicy>,
    #[serde(default)]
    pub env: HashMap<String, EnvironmentVariable>,
    pub settings: ModuleSettings
}

/// The EdgeModuleBuilder can be used to build EdgeModules when creating a modules configuration
pub struct EdgeModuleBuilder {
    module_id: Option<String>,
    version: Option<String>,
    status: Option<Status>,
    restart_policy: Option<RestartPolicy>,
    image_pull_policy: Option<ImagePullPolicy>,
    env: HashMap<String, EnvironmentVariable>,
    image: Option<String>,
    create_options: Option<serde_json::Value>,
}

impl EdgeModuleBuilder {
    /// Create a new EdgeModuleBuilder
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let edge_module_builder = EdgeModuleBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            module_id: None,
            version: None,
            status: None,
            restart_policy: None,
            image_pull_policy: None,
            env: HashMap::new(),
            image: None,
            create_options: None,
        }
    }

    /// Set the module id for the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .module_id("SomeModule");
    /// ```
    pub fn module_id<T>(mut self, module_id: T) -> Self
    where
        T: Into<String>,
    {
        self.module_id = Some(module_id.into());
        self
    }

    /// Set the version for the EdgeModule 
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .version("1.0");
    /// ```
    pub fn version<T>(mut self, version: T) -> Self
    where
        T: Into<String>,
    {
        self.version = Some(version.into());
        self
    }

    /// Set the status for the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder, Status};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .status(Status::Running);
    /// ```
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the restart policy for the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder, RestartPolicy};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .restart_policy(RestartPolicy::Always);
    /// ```
    pub fn restart_policy(mut self, restart_policy: RestartPolicy) -> Self {
        self.restart_policy = Some(restart_policy);
        self
    }
    
    /// Set the image pull policy for the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder, ImagePullPolicy};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .image_pull_policy(ImagePullPolicy::OnCreate);
    /// ```
    pub fn image_pull_policy(mut self, image_pull_policy: ImagePullPolicy) -> Self {
        self.image_pull_policy = Some(image_pull_policy);
        self
    }

    /// Add an environment variable to the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .environment_variable("variableOne", "someValue")
    ///     .environment_variable("variableTwo", "someValue");
    /// ```
    pub fn environment_variable<S,T>(mut self, key: S, value: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.env.insert(key.into(), EnvironmentVariable{value: value.into()});
        self
    }

    /// Add multiple environment variables to the EdgeModule
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let mut env_map: HashMap<String, String> = HashMap::new();
    /// env_map.insert("variableOne".to_string(), "someValue".to_string());
    /// env_map.insert("variableTwo".to_string(), "someValue".to_string());
    /// 
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .environment_variables(env_map);
    /// ```
    pub fn environment_variables(mut self, variables: HashMap<String, String>) -> Self 
    {
        for (key, value) in variables {
            self.env.insert(key, EnvironmentVariable{value});
        }
        self
    }

    /// Set the image for the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .image("some-image.acr");
    /// ```
    pub fn image<T>(mut self, image: T) -> Self
    where
        T: Into<String>,
    {
        self.image = Some(image.into());
        self
    }

    /// Set the create_options for the EdgeModule
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder};
    /// let edge_module_builder = EdgeModuleBuilder::new()
    ///     .create_options(json!({
    ///    "some": "setting"
    /// }));
    /// ```
    pub fn create_options(mut self, create_options: serde_json::Value) -> Self {
        self.create_options = Some(create_options);
        self
    }

    /// Build the EdgeModule
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{EdgeModuleBuilder, Status, RestartPolicy};
    /// let edge_module = EdgeModuleBuilder::new()
    ///     .module_id("SomeModule")
    ///     .image("some_image.acr")
    ///     .restart_policy(RestartPolicy::Always)
    ///     .status(Status::Running)
    ///     .version("1.0")
    ///     .build()
    ///     .expect("Failed to build the EdgeModule");
    ///
    /// ```
    pub fn build(self) -> Result<EdgeModule, BuilderError> {
        let module_id = match self.module_id {
            Some(val) => val,
            None => {
                return Err(BuilderError::new(BuilderErrorType::MissingValue(
                    "module_id",
                )))
            }
        };

        let version = match self.version {
            Some(val) => val,
            None => return Err(BuilderError::new(BuilderErrorType::MissingValue("version"))),
        };

        let status = match self.status {
            Some(val) => val,
            None => return Err(BuilderError::new(BuilderErrorType::MissingValue("status"))),
        };

        let restart_policy = match self.restart_policy {
            Some(val) => val,
            None => {
                return Err(BuilderError::new(BuilderErrorType::MissingValue(
                    "restart_policy",
                )))
            }
        };

        let image = match self.image {
            Some(val) => val,
            None => return Err(BuilderError::new(BuilderErrorType::MissingValue("image"))),
        };

        let module_create_options = match self.create_options {
            Some(val) => {
                match serde_json::to_string(&val) {
                    Ok(val) => Some(val),
                    Err(_) => {
                        return Err(BuilderError::new(BuilderErrorType::IncorrectValue("create_options")));
                    }
                }
            },
            None => None,
        };

        Ok(EdgeModule {
            module_id,
            version,
            module_type: "docker".to_string(),
            status,
            restart_policy,
            image_pull_policy: self.image_pull_policy,
            env: self.env,
            settings: ModuleSettings {
                image,
                create_options: module_create_options
            }
        })
    }
}

/// The registry credentials for modules configuration
#[derive(Serialize, Deserialize)]
pub struct RegistryCredential {
    username: String,
    password: String,
    address: String,
}

impl RegistryCredential {
    /// Create a new RegistryCredential
    pub fn new<S,T,U>(username: S, password: T, address: U) -> Self
    where
        S: Into<String>,
        T: Into<String>,
        U: Into<String>
    {
        Self{username: username.into(), password: password.into(), address: address.into()}
    }

    /// Get the username of the RegistryCredential
    pub fn username(&self) -> &String {
        &self.username
    }

    /// Get the password of the RegistryCredential
    pub fn password(&self) -> &String {
        &self.password
    }

    /// Get the address of the RegistryCredential
    pub fn address(&self) -> &String {
        &self.address
    }

    /// Set the username of the RegistryCredential
    pub fn set_username<S>(&mut self, username: S)
    where
        S: Into<String>
    {
        self.username = username.into();
    }
}

/// The runtime settings for the Edge Agent
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSettings {
    min_docker_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logging_options: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    registry_credentials: HashMap<String, RegistryCredential>,
}

impl RuntimeSettings {
    /// Get the minimum docker version
    pub fn min_docker_version(&self) -> &String
    {
        &self.min_docker_version
    }

    /// Get the logging options
    pub fn logging_options(&self) -> &Option<String>
    {
        &self.logging_options
    }

    /// Get the registry credentials
    pub fn registry_credentials(&self) -> &HashMap<String, RegistryCredential>
    {
        &self.registry_credentials
    }

    /// Set the minimum docker version
    pub fn set_min_docker_version<S>(&mut self, min_docker_version: S)
    where
        S: Into<String>
    {
        self.min_docker_version = min_docker_version.into();
    }

    /// Set the logging options
    pub fn set_logging_options<S>(&mut self, logging_options: Option<S>)
    where
        S: Into<String>
    {
        match logging_options {
            Some(val) => self.logging_options = Some(val.into()),
            None => self.logging_options = None
        }
    }
    
    /// Get a mutable reference to the registry credentials
    pub fn registry_credentials_mut(&mut self) -> &mut HashMap<String, RegistryCredential>
    {   
        &mut self.registry_credentials
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runtime {
    settings: RuntimeSettings,
    #[serde(rename = "type")]
    runtime_type: String,
}

impl Runtime {
    /// Get the RuntimeSettings
    pub fn settings(&self) -> &RuntimeSettings {
        &self.settings
    }

    /// Get the runtime type (always "docker")
    pub fn runtime_type(&self) -> &String 
    {
        &self.runtime_type
    }

    /// Get a mutable reference to the RuntimeSettings
    pub fn settings_mut(&mut self) -> &mut RuntimeSettings {
        &mut self.settings
    }
}

/// The settings of a module
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSettings {
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_options: Option<String>,
}

impl ModuleSettings {
    /// Get the image
    pub fn image(&self) -> &String {
        &self.image
    }

    /// Get the create options
    pub fn create_options(&self) -> &Option<String>
    {
        &self.create_options
    }

    /// Set the image 
    pub fn set_image<S>(&mut self, image: S)
    where
        S: Into<String>
    {
        self.image = image.into();
    } 

    /// Set the create options
    pub fn set_create_options(&mut self, create_options: Option<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>>
    {
        match create_options {
            Some(val) => self.create_options = Some(serde_json::to_string(&val)?),
            None => self.create_options = None
        }
        Ok(())
    }
}

/// The settings for the EdgeAgent
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeAgentSettings {
    #[serde(rename = "type")]
    runtime_type: String,
    settings: ModuleSettings,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    env: HashMap<String, EnvironmentVariable>,
}

impl EdgeAgentSettings {
    /// Get the runtime type (always "docker")
    pub fn runtime_type(&self) -> &String {
        &self.runtime_type
    }

    /// Get the settings
    pub fn settings(&self) -> &ModuleSettings {
        &self.settings
    }

    /// Get the environment variables
    pub fn env(&self) -> &HashMap<String, EnvironmentVariable>
    {
        &self.env
    }

    /// Get a mutable reference to the settings
    pub fn settings_mut(&mut self) -> &mut ModuleSettings
    {
        &mut self.settings
    }

    /// Get a mutable reference to the environment variables
    pub fn env_mut(&mut self) -> &mut HashMap<String, EnvironmentVariable>
    {
        &mut self.env
    }
}

/// The settings for the EdgeHub module
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeHubSettings {
    #[serde(rename = "type")]
    runtime_type: String,
    restart_policy: RestartPolicy,
    status: Status,
    settings: ModuleSettings,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    env: HashMap<String, EnvironmentVariable>,
}

impl EdgeHubSettings {
    /// Get the runtime type
    pub fn runtime_type(&self) -> &String {
        &self.runtime_type
    }

    /// Get the restart policy
    pub fn restart_policy(&self) -> &RestartPolicy
    {
        &self.restart_policy
    }

    /// Get the status
    pub fn status(&self) -> &Status
    {
        &self.status
    }

    /// Get the settings
    pub fn settings(&self) -> &ModuleSettings
    {
        &self.settings
    }

    /// Get the environment variables
    pub fn env(&self) -> &HashMap<String, EnvironmentVariable>
    {
        &self.env
    }

    /// Get a mutable reference to the settings
    pub fn settings_mut(&mut self) -> &mut ModuleSettings {
        &mut self.settings
    }

    /// Get a mutable reference to the environment variables
    pub fn env_mut(&mut self) -> &mut HashMap<String, EnvironmentVariable>
    {
        &mut self.env
    }
}

/// The systemmodules of the EdgeAgent properties
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemModules {
    edge_hub: EdgeHubSettings,
    edge_agent: EdgeAgentSettings,
}

impl SystemModules {
    /// Get the EdgeHub settings
    pub fn edge_hub(&self) -> &EdgeHubSettings {
        &self.edge_hub
    }

    /// Get the EdgeAgent settings
    pub fn edge_agent(&self) -> &EdgeAgentSettings {
        &self.edge_agent
    }

    /// Get a mutable reference to the EdgeHub settings
    pub fn edge_hub_mut(&mut self) -> &mut EdgeHubSettings {
        &mut self.edge_hub
    }

    /// Get a mutable reference to the EdgeAgent settings
    pub fn edge_agent_mut(&mut self) -> &mut EdgeAgentSettings {
        &mut self.edge_agent
    }
} 

/// The EdgeAgent module
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeAgent {
    schema_version: String,
    runtime: Runtime,
    system_modules: SystemModules,
    modules: HashMap<String, EdgeModule>,
}

impl EdgeAgent {
    /// Get the schema version
    pub fn schema_version(&self) -> &String {
        &self.schema_version
    }

    /// Get the runtime
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Get the system modules
    pub fn system_modules(&self) -> &SystemModules
    {
        &self.system_modules
    }

    /// Get the modules
    pub fn modules(&self) -> &HashMap<String, EdgeModule>
    {
        &self.modules
    }

    /// Get a mutable reference to the runtime
    pub fn runtime_mut(&mut self) -> &mut Runtime
    {
        &mut self.runtime
    }

    /// Get a mutable reference to the system modules
    pub fn system_modules_mut(&mut self) -> &mut SystemModules 
    {
        &mut self.system_modules
    }

    /// Get a mutable reference to the modules
    pub fn modules_mut(&mut self) -> &mut HashMap<String, EdgeModule>
    {
        &mut self.modules
    }
}

/// The store and forward configuration settings for the EdgeHub module
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreAndForwardConfiguration {
    time_to_live_secs: u64,
}

impl StoreAndForwardConfiguration {
    /// Get the time to live seconds for the store and forward configuration
    pub fn time_to_live_secs(&self) -> u64 
    {
        self.time_to_live_secs
    }

    /// Set the time to live seconds for the store and forward configuration
    pub fn set_time_to_live_secs(&mut self, time_to_live_secs: u64)
    {
        self.time_to_live_secs = time_to_live_secs;
    }
}

/// The EdgeHub module
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeHub {
    schema_version: String,
    routes: HashMap<String, String>,
    store_and_forward_configuration: StoreAndForwardConfiguration,
}

impl EdgeHub{
    /// Get the schema version
    pub fn schema_version(&self) -> &String
    {
        &self.schema_version
    }

    /// Get the routes
    pub fn routes(&self) -> &HashMap<String, String>
    {
        &self.routes
    }

    /// Get the store and forward configuration
    pub fn store_and_forward_configuration(&self) -> &StoreAndForwardConfiguration
    {
        &self.store_and_forward_configuration
    }

    /// Get a mutable reference to the routes
    pub fn routes_mut(&mut self) -> &mut HashMap<String, String>
    {
        &mut self.routes
    }

    /// Get a mutable reference to the store and forward configuration
    pub fn store_and_forward_configuration_mut(&mut self) -> &mut StoreAndForwardConfiguration
    {
        &mut self.store_and_forward_configuration
    }
}

/// The module configuration
pub struct ModulesContent {
    edge_agent: EdgeAgent,
    edge_hub: EdgeHub,
}

impl ModulesContent {
    /// Create a new module configuration
    pub fn new(edge_agent: EdgeAgent, edge_hub: EdgeHub) -> ModulesContent {
        ModulesContent {edge_agent, edge_hub}
    }

    /// Get the EdgeAgent
    pub fn edge_agent(&self) -> &EdgeAgent
    {
        &self.edge_agent
    }

    /// Get the EdgeHub
    pub fn edge_hub(&self) -> &EdgeHub
    {
        &self.edge_hub
    }

    /// Get a mutable reference to the EdgeAgent
    pub fn edge_agent_mut(&mut self) -> &mut EdgeAgent
    {
        &mut self.edge_agent
    }

    /// Get a mutable reference to the EdgeHub
    pub fn edge_hub_mut(&mut self) -> &mut EdgeHub
    {
        &mut self.edge_hub
    }
}

impl Serialize for ModulesContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModulesContent", 2)?;
        state.serialize_field(
            "$edgeAgent",
            &json!({
                "properties.desired": self.edge_agent
            }),
        )?;
        state.serialize_field(
            "$edgeHub",
            &json!({
                "properties.desired": self.edge_hub
            }),
        )?;
        state.end()
    }
}

#[derive(Default)]
pub struct ModulesContentBuilder {
    minimum_docker_version: Option<String>,
    logging_options: Option<serde_json::Value>,
    registry_credentials: HashMap<String, RegistryCredential>,
    edge_agent_env: HashMap<String, EnvironmentVariable>,
    edge_hub_env: HashMap<String, EnvironmentVariable>,
    edge_agent_image: Option<String>,
    edge_hub_image: Option<String>,
    edge_agent_create_options: Option<serde_json::Value>,
    edge_hub_create_options: Option<serde_json::Value>,
    modules: HashMap<String, EdgeModule>,
    routes: HashMap<String, String>,
    time_to_live_secs: Option<u64>,
}

impl ModulesContentBuilder {
    /// Create a new ModulesContentBuilder
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum docker version the edge device should have for this deployment
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .minimum_docker_version("v1.25");
    /// ```
    pub fn minimum_docker_version<T>(mut self, version: T) -> Self
    where
        T: Into<String>,
    {
        self.minimum_docker_version = Some(version.into());
        self
    }

    /// Add a new registry credential to the deployment manifest
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .registry_credential("some_credential", "username", "secret", "some-acr.acr");
    /// ```
    pub fn registry_credential<S,T,U,V>(mut self, name: S, username: T, password: U, address: V) -> Self
    where
        S: Into<String>,
        T: Into<String>,
        U: Into<String>,
        V: Into<String>
    {
        self.registry_credentials.insert(
            name.into(),
            RegistryCredential {
                username: username.into(),
                password: password.into(),
                address: address.into(),
            },
        );
        self
    }

    /// Add optional logging options to the deployment of the edge device
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .logging_options(json!({
    ///     "some": "options"       
    /// }));
    /// ```
    pub fn logging_options(mut self, logging_options: serde_json::Value) -> Self {
        self.logging_options = Some(logging_options.into());
        self
    }

    /// Add a route to the deployment of the edge device
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .route("one-route", "FROM /messages/modules/SomeModule/outputs/* INTO $upstream");
    /// ```
    pub fn route<S,T>(mut self, name: S, route: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.routes.insert(name.into(), route.into());
        self
    }

    /// Set the time to live of messages on the edge device in seconds
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .time_to_live_secs(10);
    /// ```
    pub fn time_to_live_secs(mut self, seconds: u64) -> Self {
        self.time_to_live_secs = Some(seconds);
        self
    }

    /// Set the image of the edge agent
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_agent_image("mcr.microsoft.com/azureiotedge-agent:1.0.9");
    /// ```
    pub fn edge_agent_image<T>(mut self, image: T) -> Self
    where
        T: Into<String>,
    {
        self.edge_agent_image = Some(image.into());
        self
    }

    /// Set the image of the edge hub
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_hub_image("mcr.microsoft.com/azureiotedge-hub:1.0.9");
    /// ```
    pub fn edge_hub_image<T>(mut self, image: T) -> Self
    where
        T: Into<String>,
    {
        self.edge_hub_image = Some(image.into());
        self
    }

    /// Set the optional create options for the edge agent
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_agent_create_options(json!({
    ///     "some": "options"       
    /// }));
    /// ```
    pub fn edge_agent_create_options(mut self, create_options: serde_json::Value) -> Self {
        self.edge_agent_create_options = Some(create_options.into());
        self
    }

    /// Set the optional create options for the edge hub
    ///
    /// # Example
    /// ```
    /// use serde_json::json;
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_hub_create_options(json!({
    ///     "some": "options"       
    /// }));
    /// ```
    pub fn edge_hub_create_options(mut self, create_options: serde_json::Value) -> Self {
        self.edge_hub_create_options = Some(create_options.into());
        self
    }

    /// Add an environment variable to the edge agent
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_agent_env("variableOne", "variable")
    ///     .edge_agent_env("variableTwo", "variable");
    /// ```
    pub fn edge_agent_env<S,T>(mut self, key: S, value: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.edge_agent_env.insert(
            key.into(),
            EnvironmentVariable {
                value: value.into(),
            },
        );
        self
    }

    /// Add an environment variable to the edge hub
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_hub_env("variableOne", "variable")
    ///     .edge_hub_env("variableTwo", "variable");
    /// ```
    pub fn edge_hub_env<S,T>(mut self, key: S, value: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.edge_hub_env.insert(
            key.into(),
            EnvironmentVariable {
                value: value.into(),
            },
        );
        self
    }

    /// Add an EdgeModule to the configuration
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder, EdgeModuleBuilder, Status, RestartPolicy};
    /// let modules_content_builder = ModulesContentBuilder::new()
    ///     .edge_module(
    ///          EdgeModuleBuilder::new()
    ///             .module_id("SomeModule")
    ///             .status(Status::Running)
    ///             .restart_policy(RestartPolicy::Always)
    ///             .image("some-image.acr")
    ///             .version("1.0")
    ///             .build().expect("Failed to build the EdgeModule")
    ///     );
    /// ```
    pub fn edge_module(mut self, edge_module: EdgeModule) -> Self {
        self.modules.insert(edge_module.module_id.clone(), edge_module);
        self
    }

    /// Build the ModulesContent
    ///
    /// # Example
    /// ```
    /// use azure_iothub_service::configuration::{ModulesContentBuilder};
    /// let modules_content = ModulesContentBuilder::new()
    ///     .edge_agent_image("mcr.microsoft.com/azureiotedge-agent:1.0.9")
    ///     .edge_hub_image("mcr.microsoft.com/azureiotedge-hub:1.0.9")
    ///     .minimum_docker_version("v1.25")
    ///     .time_to_live_secs(10)
    ///     .build()
    ///     .expect("Failed to build the ModulesContent");
    /// ```
    pub fn build(self) -> Result<ModulesContent, BuilderError> {
        let time_to_live_secs =
            self.time_to_live_secs
                .ok_or(BuilderError::new(BuilderErrorType::MissingValue(
                    "time_to_live_secs",
                )))?;

        let logging_options = match self.logging_options {
            Some(val) => {
                match serde_json::to_string(&val) {
                    Ok(stringified_json) => Some(stringified_json),
                    Err(_) => return Err(BuilderError::new(BuilderErrorType::IncorrectValue("logging_options")))
                }
            },
            None => None,
        };

        let minimum_docker_version = self.minimum_docker_version.ok_or(BuilderError::new(
            BuilderErrorType::MissingValue("minimum_docker_version"),
        ))?;

        let edgehub_image =
            self.edge_hub_image
                .ok_or(BuilderError::new(BuilderErrorType::MissingValue(
                    "edge_hub_image",
                )))?;

        let edgeagent_image =
            self.edge_agent_image
                .ok_or(BuilderError::new(BuilderErrorType::MissingValue(
                    "edge_agent_image",
                )))?;

        let edgeagent_create_options = match self.edge_agent_create_options {
            Some(val) => {
                match serde_json::to_string(&val) {
                    Ok(stringified_json) => Some(stringified_json),
                    Err(_) => return Err(BuilderError::new(BuilderErrorType::IncorrectValue("edgeagent_create_options")))
                } 
            },
            None => None,
        };

        let edgehub_create_options = match self.edge_hub_create_options {
            Some(val) => {
                match serde_json::to_string(&val) {
                    Ok(stringified_json) => Some(stringified_json),
                    Err(_) => return Err(BuilderError::new(BuilderErrorType::IncorrectValue("edgehub_create_options")))
                } 
            },
            None => None,
        };

        Ok(ModulesContent {
            edge_agent: EdgeAgent {
                schema_version: SCHEMA_VERSION.to_string(),
                runtime: Runtime {
                    settings: RuntimeSettings {
                        min_docker_version: minimum_docker_version,
                        logging_options: logging_options,
                        registry_credentials: self.registry_credentials,
                    },
                    runtime_type: RUNTIME_TYPE.to_string(),
                },
                system_modules: SystemModules {
                    edge_agent: EdgeAgentSettings {
                        runtime_type: RUNTIME_TYPE.to_string(),
                        settings: ModuleSettings {
                            create_options: edgeagent_create_options,
                            image: edgeagent_image,
                        },
                        env: self.edge_agent_env,
                    },
                    edge_hub: EdgeHubSettings {
                        settings: ModuleSettings {
                            image: edgehub_image,
                            create_options: edgehub_create_options,
                        },
                        runtime_type: RUNTIME_TYPE.to_string(),
                        restart_policy: RestartPolicy::Always,
                        status: Status::Running,
                        env: self.edge_hub_env,
                    },
                },
                modules: self.modules,
            },
            edge_hub: EdgeHub {
                schema_version: SCHEMA_VERSION.to_string(),
                routes: self.routes,
                store_and_forward_configuration: StoreAndForwardConfiguration {
                    time_to_live_secs: time_to_live_secs,
                },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::configuration::modulescontent::{
        EdgeModuleBuilder, ImagePullPolicy, ModulesContentBuilder, RestartPolicy, Status, EdgeAgent, EdgeHub,
        RUNTIME_TYPE, SCHEMA_VERSION,
    };
    use serde_json::json;
    use std::path::PathBuf;

    fn load_json_file(file_name: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/");
        d.push(file_name);

        let stringified = std::fs::read_to_string(d)?;
        Ok(serde_json::from_str(&stringified)?)
    }

    #[test]
    fn edge_module_builder_should_succeed() -> Result<(), Box<dyn std::error::Error>> {
        let create_options = json!({
            "settings": {
                "important": "setting",
                "another": "important setting"
            }
        });

        let edge_module = EdgeModuleBuilder::new()
            .module_id("SomeModule")
            .version("1.0")
            .status(Status::Running)
            .restart_policy(RestartPolicy::Never)
            .image("some-image.containerregistry.url")
            .image_pull_policy(ImagePullPolicy::Never)
            .environment_variable("great", "environment")
            .environment_variable("another", "variable")
            .create_options(create_options.clone())
            .build()
            .expect("Building the EdgeModule should have succeeded");

        assert_eq!(edge_module.module_id, "SomeModule");
        assert_eq!(edge_module.version, "1.0");
        assert_eq!(edge_module.status, Status::Running);
        assert_eq!(edge_module.restart_policy, RestartPolicy::Never);
        assert_eq!(edge_module.settings.image, "some-image.containerregistry.url");
        assert_eq!(edge_module.image_pull_policy, Some(ImagePullPolicy::Never));

        assert_eq!(
            edge_module.env.get("great").unwrap().value, "environment");

        assert_eq!(
            edge_module.env.get("another").unwrap().value, "variable");

        assert_eq!(edge_module.settings.create_options, Some(serde_json::to_string(&create_options)?));
        Ok(())
    }

    #[test]
    fn edge_agent_builder_should_succeed() -> Result<(), Box<dyn std::error::Error>> {
        let create_options = json!({
            "settings": {
                "important": "setting",
                "another": "important setting"
            }
        });

        let logging_options = json!({
            "logging": {
                "is": "important"
            }
        });

        let modules_content = ModulesContentBuilder::new()
            .minimum_docker_version("1.3.2")
            .logging_options(logging_options.clone())
            .edge_agent_image("acr_agent_image.com:1.0")
            .edge_agent_create_options(create_options.clone())
            .edge_hub_image("acr_hub_image.com:1.0")
            .edge_hub_create_options(create_options.clone())
            .time_to_live_secs(1)
            .registry_credential(
                "AcrCredential",
                "secret",
                "password",
                "some-containerregistry.com",
            )
            .registry_credential(
                "AnotherAcrCredential",
                "username",
                "secret",
                "some-containerregistry2.com",
            )
            .build()?;

        assert_eq!(modules_content.edge_agent.schema_version, SCHEMA_VERSION);
        assert_eq!(
            modules_content
                .edge_agent
                .runtime
                .settings
                .min_docker_version,
            "1.3.2"
        );
        assert_eq!(
            modules_content.edge_agent.runtime.settings.logging_options,
            Some(serde_json::to_string(&logging_options)?)
        );

        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_agent
                .runtime_type,
            RUNTIME_TYPE
        );
        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_agent
                .settings
                .image,
            "acr_agent_image.com:1.0"
        );
        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_agent
                .settings
                .create_options,
            Some(serde_json::to_string(&create_options)?)
        );

        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_hub
                .runtime_type,
            RUNTIME_TYPE
        );
        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_hub
                .settings
                .image,
            "acr_hub_image.com:1.0"
        );
        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_hub
                .settings
                .create_options,
            Some(serde_json::to_string(&create_options)?)
        );
        assert_eq!(
            modules_content
                .edge_agent
                .system_modules
                .edge_hub
                .restart_policy,
            RestartPolicy::Always
        );
        assert_eq!(
            modules_content.edge_agent.system_modules.edge_hub.status,
            Status::Running
        );
        Ok(())
    }

    #[test]
    fn modules_content_should_serialize_correctly() -> Result<(), Box<dyn std::error::Error>> {
        let test_json_file = load_json_file("configuration/modulescontent_serialization.json")?;
        let modules_content = ModulesContentBuilder::new()
            .minimum_docker_version("1.3.2")
            .logging_options(json!({"some": "option"}))
            .edge_agent_image("agent-acr.xyz:1.0")
            .edge_agent_create_options(json!({"some": "create options"}))
            .edge_hub_image("hub-acr.xyz:1.0")
            .edge_hub_create_options(json!({"some": "create options"}))
            .registry_credential("TestCred", "username", "password", "url.xyz")
            .time_to_live_secs(1)
            .build()?;

        let edge_agent_json = serde_json::to_value(modules_content)?;
        assert!(
            edge_agent_json == test_json_file,
            format!(
                "{}\n is not equal to\n {}",
                serde_json::to_string_pretty(&edge_agent_json)?,
                serde_json::to_string_pretty(&test_json_file)?
            )
        );
        Ok(())
    }

    #[test]
    fn edge_agent_should_deserialize_correctly() -> Result<(), Box<dyn std::error::Error>>
    {
        let test_json_file = load_json_file("configuration/edgeagent_deserialization.json")?;
        let edge_agent: EdgeAgent = serde_json::from_value(test_json_file)?;

        assert!(edge_agent.modules.get("SomeModule").is_some());
        Ok(())
    }

    #[test]
    fn edge_hub_should_deserialize_correctly() -> Result<(), Box<dyn std::error::Error>>
    {
        let test_json_file = load_json_file("configuration/edgehub_deserialization.json")?;
        let edge_hub: EdgeHub = serde_json::from_value(test_json_file)?;

        assert_eq!(edge_hub.routes.get("SomeRoute"), Some(&"FROM /messages/modules/SomeModule/outputs/* INTO $upstream".to_string()));
        assert_eq!(edge_hub.routes.get("AnotherRoute"), Some(&"FROM /messages/modules/AnotherModule/outputs/* INTO $upstream".to_string()));
        Ok(())
    }
}
