//! The DirectMethod module is used for invoking device and module
//! methods. However, the DirectMethod should only be constructed
//! from the iothub module.
use std::fmt;

use bytes::buf::BufExt as _;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde::de::{self, Deserializer, DeserializeOwned, Visitor, MapAccess, Unexpected};
use serde_json::json;

use crate::{IoTHubService, API_VERSION};

/// The DirectMethodResponse struct contains the response
/// from the IoT Hub when a direct method was invoked.
#[derive(Deserialize)]
pub struct DirectMethodResponse<T>
{
    pub status: u64,
    pub payload: T
}

/// The message object within an DirectMethodError
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DirectMethodErrorMessage {
    pub error_code: u64,
    pub tracking_id: String,
    pub message: String,
    pub info: serde_json::Value,
    pub timestamp_utc: String
}

/// The DirectMethodError
#[derive(Debug)]
pub struct DirectMethodError {
    pub message: DirectMethodErrorMessage,
    pub exception_message: String
}

impl<'de> Deserialize<'de> for DirectMethodError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field { Message, ExceptionMessage };

        struct DirectMethodErrorVisitor;

        impl<'de> Visitor<'de> for DirectMethodErrorVisitor {
            type Value = DirectMethodError;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct DirectMethodError")
            }

            fn visit_map<V>(self, mut map: V) -> Result<DirectMethodError, V::Error>
            where V: MapAccess<'de>
            {
                let mut message: Option<DirectMethodErrorMessage> = None;
                let mut exception_message: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Message => {
                            if message.is_some() {
                                return Err(de::Error::duplicate_field("Message"));
                            }
                            message = match serde_json::from_str(&map.next_value::<String>()?) {
                                Ok(val) => Some(val),
                                Err(err) => {
                                    println!("{}", err);
                                    return Err(de::Error::invalid_type(Unexpected::Other(&"non stringified json"), &"stringified json"));
                                }
                            };
                        },
                        Field::ExceptionMessage => {
                            if exception_message.is_some() {
                                return Err(de::Error::duplicate_field("ExceptionMessage"));
                            }
                            exception_message = Some(map.next_value()?);
                        }

                    }
                }

                let message = message.ok_or_else(|| de::Error::missing_field("Message"))?;
                let exception_message = exception_message.ok_or_else(|| de::Error::missing_field("ExceptionMessage"))?;
                Ok(DirectMethodError{ message, exception_message })
            }
        }

        const FIELDS: &'static [&'static str] = &["Message", "ExceptionMessage"];
        deserializer.deserialize_struct("DirectMethodError", FIELDS, DirectMethodErrorVisitor)
    }
}

impl std::fmt::Display for DirectMethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ error_code: {}, tracking_id: {}, message: {} }}", self.message.error_code, self.message.tracking_id, self.message.message)
    }
}

impl std::error::Error for DirectMethodError {

}

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
            let error: DirectMethodError = serde_json::from_reader(body.reader())?;

            return Err(Box::new(error));
        }

        let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
        Ok(serde_json::from_reader(body.reader())?)
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

    #[test]
    fn directmethoderror_should_deserialize() -> Result<(), Box<dyn std::error::Error>> {
        use serde_json::json;
        use crate::directmethod::DirectMethodError;

        let direct_method_error_str = "{
            \"Message\": \"{ \\\"errorCode\\\": 12345, \\\"trackingId\\\": \\\"trackingid\\\", \\\"message\\\": \\\"an error occurred\\\", \\\"info\\\": {}, \\\"timestampUtc\\\": \\\"2020-06-21T16:38:35.671+00:00\\\"}\",
            \"ExceptionMessage\": \"a great exception\"
        }";

        println!("{}", direct_method_error_str);

        let direct_method_error: DirectMethodError = serde_json::from_str(direct_method_error_str)?;
        assert_eq!(direct_method_error.message.error_code, 12345);
        assert_eq!(direct_method_error.message.tracking_id, "trackingid");
        assert_eq!(direct_method_error.message.message, "an error occurred");
        assert_eq!(direct_method_error.message.info, json!({}));
        assert_eq!(direct_method_error.message.timestamp_utc, "2020-06-21T16:38:35.671+00:00");
        assert_eq!(direct_method_error.exception_message, "a great exception");
        Ok(())
    }
}
