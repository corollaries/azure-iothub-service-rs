use serde::de::{self, Deserialize, Deserializer, MapAccess, Unexpected, Visitor};
use std::fmt;

/// Type of the builder error that occurred when building an object
#[derive(Debug, Clone)]
pub enum BuilderErrorType {
    MissingValue(&'static str),
    IncorrectValue(&'static str),
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
            BuilderErrorType::IncorrectValue(val) => write!(f, "incorrect value for {}", val),
        }
    }
}

impl std::error::Error for BuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub struct ParsingError {
    pub received_payload: String,
    pub serialization_error: Box<dyn std::error::Error>,
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Received payload: {}, got serialization error: {}",
            self.received_payload, self.serialization_error
        )
    }
}

/// The message object within an IoTHubError
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IoTHubErrorMessage {
    pub error_code: u64,
    pub tracking_id: String,
    pub message: String,
    pub info: serde_json::Value,
    pub timestamp_utc: String,
}

/// The IoTHubError
#[derive(Debug)]
pub struct IoTHubError {
    pub message: IoTHubErrorMessage,
    pub exception_message: String,
}

impl<'de> Deserialize<'de> for IoTHubError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            Message,
            ExceptionMessage,
        };

        struct IoTHubErrorVisitor;

        impl<'de> Visitor<'de> for IoTHubErrorVisitor {
            type Value = IoTHubError;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct DirectMethodError")
            }

            fn visit_map<V>(self, mut map: V) -> Result<IoTHubError, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut message: Option<IoTHubErrorMessage> = None;
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
                                    return Err(de::Error::invalid_type(
                                        Unexpected::Other(&"non stringified json"),
                                        &"stringified json",
                                    ));
                                }
                            };
                        }
                        Field::ExceptionMessage => {
                            if exception_message.is_some() {
                                return Err(de::Error::duplicate_field("ExceptionMessage"));
                            }
                            exception_message = Some(map.next_value()?);
                        }
                    }
                }

                let message = message.ok_or_else(|| de::Error::missing_field("Message"))?;
                let exception_message = exception_message
                    .ok_or_else(|| de::Error::missing_field("ExceptionMessage"))?;
                Ok(IoTHubError {
                    message,
                    exception_message,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["Message", "ExceptionMessage"];
        deserializer.deserialize_struct("DirectMethodError", FIELDS, IoTHubErrorVisitor)
    }
}

impl std::fmt::Display for IoTHubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ error_code: {}, tracking_id: {}, message: {} }}",
            self.message.error_code, self.message.tracking_id, self.message.message
        )
    }
}

impl std::error::Error for IoTHubError {}

mod tests {

    #[test]
    fn iothuberror_should_deserialize() -> Result<(), Box<dyn std::error::Error>> {
        use crate::error::IoTHubError;
        use serde_json::json;

        let direct_method_error_str = "{
            \"Message\": \"{ \\\"errorCode\\\": 12345, \\\"trackingId\\\": \\\"trackingid\\\", \\\"message\\\": \\\"an error occurred\\\", \\\"info\\\": {}, \\\"timestampUtc\\\": \\\"2020-06-21T16:38:35.671+00:00\\\"}\",
            \"ExceptionMessage\": \"a great exception\"
        }";

        println!("{}", direct_method_error_str);

        let direct_method_error: IoTHubError = serde_json::from_str(direct_method_error_str)?;
        assert_eq!(direct_method_error.message.error_code, 12345);
        assert_eq!(direct_method_error.message.tracking_id, "trackingid");
        assert_eq!(direct_method_error.message.message, "an error occurred");
        assert_eq!(direct_method_error.message.info, json!({}));
        assert_eq!(
            direct_method_error.message.timestamp_utc,
            "2020-06-21T16:38:35.671+00:00"
        );
        assert_eq!(direct_method_error.exception_message, "a great exception");
        Ok(())
    }
}
