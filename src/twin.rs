use std::collections::HashMap;

use bytes::buf::BufExt as _;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::de::{self};
use serde::{Deserialize, Deserializer};
use serde_json::json;

use crate::{error::IoTHubError, IoTHubService, API_VERSION};

#[derive(Deserialize, Debug)]
pub struct TwinError {
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "ExceptionMessage")]
    exception_message: String,
    #[serde(rename = "TrackingId")]
    tracking_id: String,
}

impl std::fmt::Display for TwinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ message: {}, exception_message: {}, tracking_id: {} }}",
            self.message, self.exception_message, self.tracking_id
        )
    }
}

impl std::error::Error for TwinError {}

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

pub struct DesiredTwin {
    contents: serde_json::Value,
}

pub struct DesiredTwinBuilder {
    desired_properties: Option<serde_json::Value>,
    desired_tags: HashMap<String, String>,
}

impl DesiredTwinBuilder {
    pub fn new() -> Self {
        DesiredTwinBuilder {
            desired_properties: None,
            desired_tags: HashMap::new(),
        }
    }

    pub fn add_tag<T>(mut self, tag_name: T, tag_value: T) -> Self
    where
        T: Into<String>,
    {
        self.desired_tags.insert(tag_name.into(), tag_value.into());
        self
    }

    pub fn properties(mut self, desired_properties: serde_json::Value) -> Self {
        self.desired_properties = Some(desired_properties);
        self
    }

    pub fn build(self) -> DesiredTwin {
        DesiredTwin {
            contents: json!({
                "propeties": {
                    "desired": self.desired_properties.unwrap_or(json!({}))
                },
                "tags": self.desired_tags
            }),
        }
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
        desired_twin: DesiredTwin,
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
            .body(Body::from(serde_json::to_string(&desired_twin.contents)?))?;

        let response = client.request(request).await?;
        if !response.status().is_success() {
            let body = hyper::body::to_bytes(response).await?;
            let twin_error: TwinError = serde_json::from_slice(&body)?;
            return Err(Box::new(twin_error));
        }

        let body = hyper::body::to_bytes(response).await?;
        Ok(serde_json::from_slice(&body)?)
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
        desired_twin: DesiredTwin,
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

        self.update_twin(uri, Method::PATCH, desired_twin).await
    }

    pub async fn update_module_twin<S, T>(
        &self,
        device_id: S,
        module_id: T,
        desired_twin: DesiredTwin,
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

        self.update_twin(uri, Method::PATCH, desired_twin).await
    }

    pub async fn replace_device_twin<T>(
        self,
        device_id: T,
        desired_twin: DesiredTwin,
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

        self.update_twin(uri, Method::PUT, desired_twin).await
    }

    pub async fn replace_module_twin<S, T>(
        &self,
        device_id: S,
        module_id: T,
        desired_twin: DesiredTwin,
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

        self.update_twin(uri, Method::PUT, desired_twin).await
    }
}
