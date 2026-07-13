//! This module contains functions with requests to test server

use net::{
    objects::{Health, Overview, ParseErrorDTO, SqlQueryOutput},
    requests::SqlQueryRequest,
};

pub async fn send_ping(url: &str) -> String {
    let res = reqwest::Client::new()
        .get(format!("http://{}/ping", url))
        .send()
        .await
        .expect("Error while sending request");
    res.text().await.expect("Error while parsing output")
}

pub async fn send_health(url: &str) -> Health {
    let res = reqwest::Client::new()
        .get(format!("http://{}/health", url))
        .send()
        .await
        .expect("Error while sending request");
    res.json::<Health>()
        .await
        .expect("Error while parsing output")
}

pub async fn send_overview(url: &str) -> Overview {
    let res = reqwest::Client::new()
        .get(format!("http://{}/v1/overview", url))
        .send()
        .await
        .expect("Error while sending request");
    res.json::<Overview>()
        .await
        .expect("Error while parsing output")
}

pub async fn send_query(url: &str, query: &str) -> Result<SqlQueryOutput, ParseErrorDTO> {
    let res = reqwest::Client::new()
        .post(format!("http://{}/v1/query", url))
        .json(&SqlQueryRequest::new(query.to_owned()))
        .send()
        .await
        .expect("Error while sending request");
    println!("{}", res.status());
    if res.status().is_success() {
        Ok(res
            .json::<SqlQueryOutput>()
            .await
            .expect("Error while parsing output"))
    } else {
        Err(res
            .json::<ParseErrorDTO>()
            .await
            .expect("Error while parsing output"))
    }
}
