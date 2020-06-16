use std::collections::HashMap;

use bytes::buf::BufExt as _;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::de::{self};
use serde::{Deserialize, Deserializer};
use serde_json::json;

use crate::{IoTHubService, API_VERSION};

/// AuthenticationType of a module or device
pub enum AuthenticationType {
    Certificate,
    Authority,
    None,
    SAS,
    SelfSigned,
}

impl<'de> Deserialize<'de> for AuthenticationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "certificate" => Ok(AuthenticationType::Certificate),
            "sas" => Ok(AuthenticationType::SAS),
            "Authority" => Ok(AuthenticationType::Authority),
            "selfSigned" => Ok(AuthenticationType::SelfSigned),
            "none" => Ok(AuthenticationType::None),
            _ => Err(de::Error::custom(format!("Expected status to be 'certificate','sas','Authority','selfSigned' or 'none' but received: {}", s))),
        }
    }
}

/// The connection state of a module or device
pub enum ConnectionState {
    Connected,
    Disconnected,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConnectionState::Connected => write!(f, "connected"),
            ConnectionState::Disconnected => write!(f, "disconnected"),
        }
    }
}

impl<'de> Deserialize<'de> for ConnectionState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Connected" => Ok(ConnectionState::Connected),
            "Disconnected" => Ok(ConnectionState::Disconnected),
            _ => Err(de::Error::custom(format!(
                "Expected status to be 'Connected' or 'Disconnected' but received: {}",
                s
            ))),
        }
    }
}

/// Device or module status
pub enum Status {
    Disabled,
    Enabled,
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "disabled" => Ok(Status::Disabled),
            "enabled" => Ok(Status::Enabled),
            _ => Err(de::Error::custom(format!(
                "Expected status to be enabled or disabled but received: {}",
                s
            ))),
        }
    }
}

#[derive(Deserialize)]
pub struct DeviceCapabilities {
    #[serde(rename = "iotEdge")]
    pub iotedge: bool,
}

#[derive(Deserialize)]
pub struct X509ThumbPrint {
    pub primary_thumbprint: Option<String>,
    pub secondary_thumbprint: Option<String>,
}

#[derive(Deserialize)]
pub struct TwinProperties {
    pub desired: serde_json::Value,
    pub reported: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTwin {
    pub authentication_type: AuthenticationType,
    pub capabilities: DeviceCapabilities,
    pub cloud_to_device_message_count: i64,
    pub connection_state: ConnectionState,
    pub device_etag: String,
    pub device_id: String,
    pub device_scope: Option<String>,
    pub etag: String,
    pub last_activity_time: String,
    pub parent_scopes: Option<Vec<String>>,
    pub properties: TwinProperties,
    pub status: Status,
    pub status_reason: Option<String>,
    pub status_update_time: String,
    pub tags: HashMap<String, String>,
    pub version: i64,
    pub x509_thumbprint: X509ThumbPrint,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleTwin {
    pub authentication_type: AuthenticationType,
    pub cloud_to_device_message_count: i64,
    pub connection_state: ConnectionState,
    pub device_etag: String,
    pub device_id: String,
    pub etag: String,
    pub last_activity_time: String,
    pub module_id: String,
    pub properties: TwinProperties,
    pub status: Status,
    pub status_update_time: String,
    pub version: i64,
    pub x509_thumbprint: X509ThumbPrint,
}

pub struct TwinBuilder {
    desired_properties: Option<serde_json::Value>,
    desired_tags: HashMap<String, String>,
}

impl TwinBuilder {
    pub fn new(self) -> Self {
        TwinBuilder {
            desired_properties: None,
            desired_tags: HashMap::new(),
        }
    }

    pub fn add_tag<T>(mut self, tag_name: T, tag_value: T)
    where
        T: Into<String>,
    {
        self.desired_tags.insert(tag_name.into(), tag_value.into());
    }

    pub fn properties(mut self, desired_properties: serde_json::Value) {
        self.desired_properties = Some(desired_properties);
    }

    pub fn build(self) -> serde_json::Value {
        json!({
            "properties": {
                "desired": self.desired_properties.unwrap_or(json!({}))
            },
            "tags": self.desired_tags
        })
    }
}

pub struct TwinManager<'a> {
    iothub_service: &'a IoTHubService,
}

impl<'a> TwinManager<'a> {
    pub fn new(iothub_service: &'a IoTHubService) -> Self {
        TwinManager { iothub_service }
    }

    async fn get_twin<T>(&self, uri: String) -> Result<T, Box<dyn std::error::Error>>
    where
        for<'de> T: Deserialize<'de>,
    {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let request = Request::builder()
            .uri(uri)
            .method(Method::GET)
            .header("Authorization", &self.iothub_service.sas_token)
            .header("Content-Type", "application/json")
            .body(Body::empty())?;

        let response = client.request(request).await?;
        let body = hyper::body::aggregate(response).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }

    async fn update_twin<T>(
        &self,
        uri: String,
        method: Method,
        twin_patch: serde_json::Value,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        for<'de> T: Deserialize<'de>,
    {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let request = Request::builder()
            .uri(uri)
            .method(method)
            .header("Authorization", &self.iothub_service.sas_token)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&twin_patch)?))?;

        let response = client.request(request).await?;
        let body = hyper::body::aggregate(response).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }

    pub async fn get_device_twin<T>(
        self,
        device_id: T,
    ) -> Result<DeviceTwin, Box<dyn std::error::Error>>
    where
        T: Into<String>,
    {
        let uri = format!(
            "https://{}.azure-devices.net/twins/{}?api-version={}",
            self.iothub_service.iothub_name,
            device_id.into(),
            API_VERSION
        );

        self.get_twin(uri).await
    }

    pub async fn get_module_twin<S, T>(
        &self,
        device_id: S,
        module_id: T,
    ) -> Result<ModuleTwin, Box<dyn std::error::Error>>
    where
        S: Into<String>,
        T: Into<String>,
    {
        let uri = format!(
            "https://{}.azure-devices.net/twins/{}/modules/{}?api-version={}",
            self.iothub_service.iothub_name,
            device_id.into(),
            module_id.into(),
            API_VERSION
        );

        self.get_twin(uri).await
    }

    pub async fn update_device_twin<T>(
        &self,
        device_id: T,
        twin_patch: serde_json::Value,
    ) -> Result<DeviceTwin, Box<dyn std::error::Error>>
    where
        T: Into<String>,
    {
        let uri = format!(
            "https://{}.azure-devices.net/twins/{}?api-version={}",
            self.iothub_service.iothub_name,
            device_id.into(),
            API_VERSION
        );

        self.update_twin(uri, Method::PATCH, twin_patch).await
    }

    pub async fn update_module_twin<S, T>(
        &self,
        device_id: S,
        module_id: T,
        twin_patch: serde_json::Value,
    ) -> Result<ModuleTwin, Box<dyn std::error::Error>>
    where
        S: Into<String>,
        T: Into<String>,
    {
        let uri = format!(
            "https://{}.azure-devices.net/twins/{}/modules/{}?api-version={}",
            self.iothub_service.iothub_name,
            device_id.into(),
            module_id.into(),
            API_VERSION
        );

        self.update_twin(uri, Method::PATCH, twin_patch).await
    }

    pub async fn replace_device_twin<T>(
        self,
        device_id: T,
        twin_patch: serde_json::Value,
    ) -> Result<DeviceTwin, Box<dyn std::error::Error>>
    where
        T: Into<String>,
    {
        let uri = format!(
            "https://{}.azure-devices.net/twins/{}?api-version={}",
            self.iothub_service.iothub_name,
            device_id.into(),
            API_VERSION
        );

        self.update_twin(uri, Method::PUT, twin_patch).await
    }

    pub async fn replace_module_twin<S, T>(
        &self,
        device_id: S,
        module_id: T,
        twin_patch: serde_json::Value,
    ) -> Result<ModuleTwin, Box<dyn std::error::Error>>
    where
        S: Into<String>,
        T: Into<String>,
    {
        let uri = format!(
            "https://{}.azure-devices.net/twins/{}/modules/{}?api-version={}",
            self.iothub_service.iothub_name,
            device_id.into(),
            module_id.into(),
            API_VERSION
        );

        self.update_twin(uri, Method::PUT, twin_patch).await
    }
}
