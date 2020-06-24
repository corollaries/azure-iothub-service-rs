//! The DirectMethod module is used for invoking device and module
//! methods. However, the DirectMethod should only be constructed
//! from the iothub module.
use std::fmt;

use bytes::buf::BufExt as _;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;

use crate::error::{IoTHubError, ParsingError};
use crate::{IoTHubService, API_VERSION};

/// The DirectMethodResponse struct contains the response
/// from the IoT Hub when a direct method was invoked.
#[derive(Deserialize)]
pub struct DirectMethodResponse<T> {
    pub status: u64,
    pub payload: T,
}

#[derive(Debug)]
pub enum DirectMethodError {
    IoTHubError(IoTHubError),
    ParsingError(ParsingError),
}

impl std::fmt::Display for DirectMethodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectMethodError::IoTHubError(val) => write!(f, "{}", val),
            DirectMethodError::ParsingError(val) => write!(f, "{}", val),
        }
    }
}

impl std::error::Error for DirectMethodError {}

/// The DirectMethod struct contains all neccessary properties
/// to be able to invoke the method.
pub struct DirectMethod<'a> {
    iothub_service: &'a IoTHubService,
    device_id: String,
    module_id: Option<String>,
    method_name: String,
    connect_time_out: u64,
    response_time_out: u64,
}

impl<'a> DirectMethod<'a> {
    /// Create a new DirectMethod
    pub(crate) fn new(
        iothub_service: &'a IoTHubService,
        device_id: String,
        module_id: Option<String>,
        method_name: String,
        response_time_out: u64,
        connect_time_out: u64,
    ) -> Self {
        DirectMethod {
            iothub_service,
            device_id,
            module_id,
            method_name,
            connect_time_out,
            response_time_out,
        }
    }

    /// Invoke the DirectMethod
    ///
    /// Either a module method, or device method is invoked based on the
    /// way the DirectMethod was created. On invocation a DirectMethodResponse
    /// is returned. This does not mean the invocation was successfull. The status
    /// code within the DirectMethodResponse should still be verified.
    ///
    /// # Examples
    /// ```
    /// # use serde_json::json;
    /// use azure_iothub_service::IoTHubService;
    ///
    /// let service = IoTHubService::from_sas_token("some-iot-hub", "sas_token");
    /// let great_method = service.create_device_method(
    ///    "SomeDeviceId",
    ///    "GreatMethod",
    ///    100,
    ///    60
    /// );
    ///
    /// great_method.invoke::<serde_json::Value>(json!({"hello": "world"}));
    /// ```
    pub async fn invoke<T: DeserializeOwned>(
        &self,
        payload: serde_json::Value,
    ) -> Result<DirectMethodResponse<T>, Box<dyn std::error::Error>> {
        match &self.module_id {
            Some(module_id_value) => {
                let uri = format!(
                    "https://{}.azure-devices.net/twins/{}/modules/{}/methods?api-version={}",
                    self.iothub_service.iothub_name, self.device_id, module_id_value, API_VERSION
                );
                Ok(self.invoke_method(&uri, payload.into()).await?)
            }
            None => {
                let uri = format!(
                    "https://{}.azure-devices.net/twins/{}/methods?api-version={}",
                    self.iothub_service.iothub_name, self.device_id, API_VERSION
                );
                Ok(self.invoke_method(&uri, payload.into()).await?)
            }
        }
    }

    /// Helper method for invoking the method
    async fn invoke_method<T: DeserializeOwned>(
        &self,
        uri: &str,
        payload: serde_json::Value,
    ) -> Result<DirectMethodResponse<T>, Box<dyn std::error::Error>> {
        let json_payload = json!({
            "connectTimeoutInSeconds": self.connect_time_out,
            "methodName": self.method_name,
            "payload": payload,
            "responseTimeoutInSeconds": self.response_time_out,
        });

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let request = Request::builder()
            .uri(uri)
            .method(Method::POST)
            .header("Authorization", &self.iothub_service.sas_token)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_payload)?))?;

        let mut response = client.request(request).await?;
        if !response.status().is_success() {
            let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
            let error: IoTHubError = serde_json::from_reader(body.reader())?;
            return Err(Box::new(DirectMethodError::IoTHubError(error)));
        }

        let body = hyper::body::to_bytes(response.body_mut()).await?;
        let result: serde_json::Result<DirectMethodResponse<T>> = serde_json::from_slice(&body);
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                let body_string = String::from_utf8_lossy(&body);
                Err(Box::new(DirectMethodError::ParsingError(ParsingError {
                    received_payload: body_string.to_string(),
                    serialization_error: Box::new(err),
                })))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::IoTHubService;

    #[test]
    fn directmethod_new_should_succeed() {
        use crate::directmethod::DirectMethod;

        let service: IoTHubService = IoTHubService::from_sas_token("test", "test");
        let direct_method = DirectMethod::new(
            &service,
            "SomeDevice".to_string(),
            None,
            "GreatMethod".to_string(),
            20,
            10,
        );
        assert_eq!(direct_method.device_id, "SomeDevice");
        assert_eq!(direct_method.module_id, None);
        assert_eq!(direct_method.method_name, "GreatMethod");
        assert_eq!(direct_method.connect_time_out, 10);
        assert_eq!(direct_method.response_time_out, 20);
    }
}
