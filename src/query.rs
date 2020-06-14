use bytes::buf::BufExt as _;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::json;
use std::str::FromStr;
use std::vec::Vec;

use crate::{IoTHubService, API_VERSION};

pub struct Query<'a> {
    iothub_service: &'a IoTHubService,
    query: String,
}

impl<'a> Query<'a> {
    pub async fn execute(self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let uri = format!(
            "https://{}.azure-devices.net/devices/query?api-version={}",
            self.iothub_service.iothub_name, API_VERSION
        );

        let json_payload = json!({
            "query": self.query,
        });

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let request = Request::builder()
            .uri(uri)
            .method(Method::POST)
            .header("Authorization", &self.iothub_service.sas_token)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_payload)?))?;

        let response = client.request(request).await?;
        let body = hyper::body::aggregate(response).await?;
        Ok(serde_json::from_reader(body.reader())?)
    }
}

pub struct QueryBuilder<'a> {
    iothub_service: &'a IoTHubService,
    select: Option<String>,
    from: Option<String>,
    and_where: Option<String>,
    group_by: Option<String>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(iothub_service: &'a IoTHubService) -> Self {
        QueryBuilder {
            iothub_service,
            select: None,
            from: None,
            and_where: None,
            group_by: None,
        }
    }

    pub fn select<T>(mut self, select_query: T) -> Self
    where
        T: Into<String>,
    {
        self.select = Some(select_query.into());
        self
    }

    pub fn from<T>(mut self, from_query: T) -> Self
    where
        T: Into<String>,
    {
        self.from = Some(from_query.into());
        self
    }

    pub fn and_where<T>(mut self, where_query: T) -> Self
    where
        T: Into<String>,
    {
        self.and_where = Some(where_query.into());
        self
    }

    pub fn group_by<T>(mut self, group_by_query: T) -> Self
    where
        T: Into<String>,
    {
        self.group_by = Some(group_by_query.into());
        self
    }

    pub fn build(self) -> Query<'a> {
        let mut query: String = "".to_string();

        match self.select {
            Some(select_query) => {
                query = [query, "SELECT ".to_string(), select_query].concat();
            }
            None => {}
        }

        match self.from {
            Some(from_query) => {
                query = [query, " FROM ".to_string(), from_query].concat();
            }
            None => {}
        }

        match self.and_where {
            Some(filter_query) => {
                query = [query, " WHERE ".to_string(), filter_query].concat();
            }
            None => {}
        }

        match self.group_by {
            Some(group_by_query) => {
                query = [query, " GROUP BY ".to_string(), group_by_query].concat();
            }
            None => {}
        }

        Query {
            iothub_service: self.iothub_service,
            query,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::IoTHubService;

    #[test]
    fn querybuilder_success() {
        use crate::QueryBuilder;
        let iothub_service = IoTHubService {
            iothub_name: "test".to_string(),
            sas_token: "test".to_string(),
        };
        let query = QueryBuilder::new(&iothub_service)
            .select("properties.something")
            .from("modules")
            .and_where("x == something")
            .group_by("something")
            .build();

        let expected_query =
            "SELECT properties.something FROM modules WHERE x == something GROUP BY something"
                .to_string();
        assert_eq!(expected_query, query.query);
    }
}
