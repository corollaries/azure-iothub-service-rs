use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_json::json;
use std::collections::HashMap;

/// The schema version of the modulescontent
const SCHEMA_VERSION: &str = "1.0";

/// The runtime type for the containers
const RUNTIME_TYPE: &str = "docker";

/// Type of the builder error that occurred when building an object
#[derive(Debug, Clone)]
pub enum BuilderErrorType {
    MissingValue(&'static str),
}

/// BuilderError struct that contains the type of error that occurred
/// when using a builder
#[derive(Debug, Clone)]
pub struct BuilderError {
    error_type: BuilderErrorType,
}

impl BuilderError {
    /// Create a new BuilderError struct
    pub fn new(error_type: BuilderErrorType) -> Self {
        BuilderError { error_type }
    }
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.error_type {
            BuilderErrorType::MissingValue(val) => write!(f, "missing field {}", val),
        }
    }
}

impl std::error::Error for BuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// The status of a module, either Running or Stopped
#[derive(Debug, PartialEq)]
pub enum Status {
    Running,
    Stopped,
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Status::Running => serializer.serialize_str("running"),
            Status::Stopped => serializer.serialize_str("stopped"),
        }
    }
}

/// The restart policy of a module
#[derive(Debug, PartialEq)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    OnUnhealthy,
    Always,
}

impl Serialize for RestartPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RestartPolicy::OnFailure => serializer.serialize_str("on-failure"),
            RestartPolicy::OnUnhealthy => serializer.serialize_str("on-unhealthy"),
            RestartPolicy::Always => serializer.serialize_str("always"),
            RestartPolicy::Never => serializer.serialize_str("never"),
        }
    }
}

/// The image pull policy of a module
#[derive(Debug, PartialEq)]
pub enum ImagePullPolicy {
    OnCreate,
    Never,
}

impl Serialize for ImagePullPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ImagePullPolicy::OnCreate => serializer.serialize_str("on-create"),
            ImagePullPolicy::Never => serializer.serialize_str("never"),
        }
    }
}

#[derive(Serialize)]
struct EnvironmentVariable {
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeModule {
    #[serde(skip_serializing)]
    pub module_id: String,
    pub version: String,
    pub module_type: &'static str,
    pub status: Status,
    pub restart_policy: RestartPolicy,
    pub image_pull_policy: Option<ImagePullPolicy>,
    pub env: HashMap<String, String>,
    pub image: String,
    pub create_options: Option<serde_json::Value>,
}

struct EdgeModuleBuilder {
    module_id: Option<String>,
    version: Option<String>,
    status: Option<Status>,
    restart_policy: Option<RestartPolicy>,
    image_pull_policy: Option<ImagePullPolicy>,
    env: HashMap<String, String>,
    image: Option<String>,
    create_options: Option<serde_json::Value>,
}

impl EdgeModuleBuilder {
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

    pub fn module_id<T>(mut self, module_id: T) -> Self
    where
        T: Into<String>,
    {
        self.module_id = Some(module_id.into());
        self
    }

    pub fn version<T>(mut self, version: T) -> Self
    where
        T: Into<String>,
    {
        self.version = Some(version.into());
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    pub fn restart_policy(mut self, restart_policy: RestartPolicy) -> Self {
        self.restart_policy = Some(restart_policy);
        self
    }

    pub fn image_pull_policy(mut self, image_pull_policy: ImagePullPolicy) -> Self {
        self.image_pull_policy = Some(image_pull_policy);
        self
    }

    pub fn environment_variable<T>(mut self, key: T, value: T) -> Self
    where
        T: Into<String>,
    {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn image<T>(mut self, image: T) -> Self
    where
        T: Into<String>,
    {
        self.image = Some(image.into());
        self
    }

    pub fn create_options(mut self, create_options: serde_json::Value) -> Self {
        self.create_options = Some(create_options);
        self
    }

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

        Ok(EdgeModule {
            module_id,
            version,
            module_type: "docker",
            status,
            restart_policy,
            image_pull_policy: self.image_pull_policy,
            env: self.env,
            image,
            create_options: self.create_options,
        })
    }
}

#[derive(Serialize)]
struct RegistryCredential {
    username: String,
    password: String,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeSettings {
    min_docker_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    logging_options: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    registry_credentials: HashMap<String, RegistryCredential>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Runtime {
    settings: RuntimeSettings,
    #[serde(rename = "type")]
    runtime_type: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ModuleSettings {
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_options: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeAgentSettings {
    #[serde(rename = "type")]
    runtime_type: String,
    settings: ModuleSettings,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    env: HashMap<String, EnvironmentVariable>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeHubSettings {
    #[serde(rename = "type")]
    runtime_type: String,
    restart_policy: RestartPolicy,
    status: Status,
    settings: ModuleSettings,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    env: HashMap<String, EnvironmentVariable>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemModules {
    edge_hub: EdgeHubSettings,
    edge_agent: EdgeAgentSettings,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeAgent {
    schema_version: String,
    runtime: Runtime,
    system_modules: SystemModules,
    modules: HashMap<String, EdgeModule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StoreAndForwardConfiguration {
    time_to_live_secs: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeHub {
    schema_version: String,
    routes: HashMap<String, String>,
    store_and_forward_configuration: StoreAndForwardConfiguration,
}
pub struct ModulesContent {
    edge_agent: EdgeAgent,
    edge_hub: EdgeHub,
}

impl Serialize for ModulesContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModulesContent", 2)?;
        state.serialize_field("$edgeAgent", &json!({
            "properties.desired": self.edge_agent
        }))?;
        state.serialize_field("$edgeHub", &json!({
            "properties.desired": self.edge_hub
        }))?;
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum docker version the edge device should have for this deployment
    pub fn minimum_docker_version<T>(mut self, version: T) -> Self
    where
        T: Into<String>,
    {
        self.minimum_docker_version = Some(version.into());
        self
    }

    /// Add a new registry credential to the deployment manifest
    pub fn registry_credential<T>(mut self, name: T, username: T, password: T, url: T) -> Self
    where
        T: Into<String>,
    {
        self.registry_credentials.insert(
            name.into(),
            RegistryCredential {
                username: username.into(),
                password: password.into(),
                url: url.into(),
            },
        );
        self
    }

    /// Add optional logging options to the deployment of the edge device
    pub fn logging_options(mut self, logging_options: serde_json::Value) -> Self {
        self.logging_options = Some(logging_options.into());
        self
    }

    /// Add a route to the deployment of the edge device
    pub fn route<T>(mut self, name: T, route: T) -> Self
    where
        T: Into<String>,
    {
        self.routes.insert(name.into(), route.into());
        self
    }

    /// Set the time to live of messages on the edge device in seconds
    pub fn time_to_live_secs(mut self, seconds: u64) -> Self {
        self.time_to_live_secs = Some(seconds);
        self
    }

    /// Set the image of the edge agent
    pub fn edge_agent_image<T>(mut self, image: T) -> Self
    where
        T: Into<String>,
    {
        self.edge_agent_image = Some(image.into());
        self
    }

    /// Set the image of the edge hub
    pub fn edge_hub_image<T>(mut self, image: T) -> Self
    where
        T: Into<String>,
    {
        self.edge_hub_image = Some(image.into());
        self
    }

    /// Set the optional create options for the edge agent
    pub fn edge_agent_create_options(mut self, create_options: serde_json::Value) -> Self {
        self.edge_agent_create_options = Some(create_options.into());
        self
    }

    /// Set the optional create options for the edge hub
    pub fn edge_hub_create_options(mut self, create_options: serde_json::Value) -> Self {
        self.edge_hub_create_options = Some(create_options.into());
        self
    }

    /// Add an environment variable to the edge agent
    pub fn edge_agent_env<T>(mut self, key: T, value: T) -> Self
    where
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
    pub fn edge_hub_env<T>(mut self, key: T, value: T) -> Self
    where
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

    /// Build the ModulesContent 
    pub fn build(self) -> Result<ModulesContent, Box<dyn std::error::Error>> {
        let time_to_live_secs =
            self.time_to_live_secs
                .ok_or(BuilderError::new(BuilderErrorType::MissingValue(
                    "time_to_live_secs",
                )))?;

        let logging_options = match self.logging_options {
            Some(val) => Some(serde_json::to_string(&val)?),
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
            Some(val) => Some(serde_json::to_string(&val)?),
            None => None,
        };

        let edgehub_create_options = match self.edge_hub_create_options {
            Some(val) => Some(serde_json::to_string(&val)?),
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
                    runtime_type: RUNTIME_TYPE.to_string()
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
        EdgeModuleBuilder, ImagePullPolicy, ModulesContentBuilder, RestartPolicy, Status,
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
    fn edge_module_builder_should_succeed() {
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
        assert_eq!(edge_module.image, "some-image.containerregistry.url");
        assert_eq!(edge_module.image_pull_policy, Some(ImagePullPolicy::Never));

        assert_eq!(
            edge_module.env.get("great"),
            Some(&"environment".to_string())
        );
        assert_eq!(
            edge_module.env.get("another"),
            Some(&"variable".to_string())
        );

        assert_eq!(edge_module.create_options, Some(create_options));
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

        assert_eq!(
            modules_content.edge_agent.schema_version,
            SCHEMA_VERSION
        );
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
    fn edge_agent_should_serialize_correctly() -> Result<(), Box<dyn std::error::Error>> {
        let test_json_file = load_json_file("configuration/edgeagent_serialization.json")?;
        let edge_agent = ModulesContentBuilder::new()
            .minimum_docker_version("1.3.2")
            .logging_options(json!({"some": "option"}))
            .edge_agent_image("agent-acr.xyz:1.0")
            .edge_agent_create_options(json!({"some": "create options"}))
            .edge_hub_image("hub-acr.xyz:1.0")
            .edge_hub_create_options(json!({"some": "create options"}))
            .registry_credential("TestCred", "username", "password", "url.xyz")
            .time_to_live_secs(1)
            .build()?;

        let edge_agent_json = serde_json::to_value(edge_agent)?;
        assert!(edge_agent_json == test_json_file, format!("{}\n is not equal to\n {}", serde_json::to_string_pretty(&edge_agent_json)?, serde_json::to_string_pretty(&test_json_file)?));
        Ok(())
    }
}
