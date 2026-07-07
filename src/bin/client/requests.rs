use std::{collections::HashMap, sync::Arc};

use query::{QueryError, QueryOutput};
use serde::{Deserialize, Serialize};
use storage::common_types::{DataValue, Schema};
#[derive(Debug, Clone)]
pub enum FetchError {
    ReqwestError(Arc<reqwest::Error>),
    IOError(Arc<std::io::Error>),
    ParsingError,
    QueryError(String),
}

impl From<std::io::Error> for FetchError {
    fn from(v: std::io::Error) -> Self {
        Self::IOError(Arc::new(v))
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(v: reqwest::Error) -> Self {
        Self::ReqwestError(Arc::new(v))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overview {
    overview: HashMap<String, Schema>,
}

impl Overview {
    pub fn tables(&self) -> &HashMap<String, Schema> {
        &self.overview
    }
}

pub async fn fetch_overview(url: String) -> Result<Overview, FetchError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.spawn(async move {
        let client = reqwest::Client::builder().build()?;
        let response = client
            .get(format!("{}{}", url, "/v1/overview"))
            .send()
            .await?;
        Ok(response.json::<Overview>().await?)
    })
    .await
    .unwrap()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendQuery {
    sql: String,
}
pub async fn send_query(url: String, query: String) -> Result<(), FetchError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.spawn(async move {
        let client = reqwest::Client::builder().build()?;
        let response = client
            .post(format!("{}/v1/query", url))
            .json(&SendQuery { sql: query })
            .send()
            .await?;
        log::debug!("{:?}", response);
        let _output = response
            .json::<Vec<Result<QueryOutput, QueryError>>>() //TODO: Add some sort of output for errors
            .await?;
        Ok(())
    })
    .await
    .unwrap()
}
pub async fn fetch_rows(url: String, table: String) -> Result<Vec<Vec<DataValue>>, FetchError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.spawn(async move {
        let client = reqwest::Client::builder().build()?;
        let response = client
            .post(format!("{}/v1/query", url))
            .json(&SendQuery {
                sql: format!("SELECT * FROM {}", table),
            })
            .send()
            .await?;
        log::debug!("{:?}", response);
        let output = response
            .json::<Vec<Result<QueryOutput, QueryError>>>()
            .await?;
        let result = if let Ok(QueryOutput::Rows(r)) = output[0].clone() {
            log::info!("Received {} rows", r.len());
            r
        } else {
            return Err(FetchError::ParsingError);
        };
        Ok(result)
    })
    .await
    .unwrap()
}
pub async fn ping(url: String) -> Result<(), FetchError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.spawn(async move {
        let client = reqwest::Client::builder().build()?;
        let response = client.get(format!("{}{}", url, "/ping")).send().await?;
        if response.text().await? != "pong" {
            return Err(FetchError::ParsingError);
        }
        Ok(())
    })
    .await
    .unwrap()
}
