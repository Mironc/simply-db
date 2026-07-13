use std::sync::Arc;

use net::objects::{Overview, SqlQueryOutput};
use query::QueryOutput;
use storage::common_types::DataValue;
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
pub async fn send_query(url: String, query: String) -> Result<(), FetchError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.spawn(async move {
        let client = reqwest::Client::builder().build()?;
        let res = client
            .post(format!("{}/v1/query", url))
            .json(&net::requests::SqlQueryRequest::new(query))
            .send()
            .await?;
        log::debug!("{:?}", res);
        if res.status() == 200 {
            let _output = res.json::<SqlQueryOutput>().await?;
        }
        Ok(())
    })
    .await
    .unwrap()
}
pub async fn fetch_rows(url: String, table: String) -> Result<Vec<Vec<DataValue>>, FetchError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.spawn(async move {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/query", url))
            .json(&net::requests::SqlQueryRequest::new(format!(
                "SELECT * FROM {}",
                table
            )))
            .send()
            .await?;
        log::debug!("{:?}", response.error_for_status_ref());
        let output = response.json::<SqlQueryOutput>().await?;
        let result = if let Ok(QueryOutput::Rows(r)) = output.output()[0].clone() {
            log::info!("Received {} rows", r.len());
            r
        } else {
            log::error!("Row fetching query returned not rows");
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
        let client = reqwest::Client::new();
        let response = client.get(format!("{}{}", url, "/ping")).send().await?;
        if response.text().await? != "pong" {
            return Err(FetchError::ParsingError);
        }
        Ok(())
    })
    .await
    .unwrap()
}
