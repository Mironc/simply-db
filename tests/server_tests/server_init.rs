use net::{
    objects::{Health, Overview, ParseErrorDTO, SqlQueryOutput},
    requests::SqlQueryRequest,
};
use tokio::net::TcpListener;

const BASE_IP: &'static str = "127.0.0.1";

pub struct ServerInstance {
    process: std::process::Child,
    listen_ip: String,
}

impl ServerInstance {
    pub async fn new() -> std::io::Result<Self> {
        let exe_path = env!("CARGO_BIN_EXE_server");
        let listener = TcpListener::bind(format!("{}:0", BASE_IP)).await.unwrap();
        let assigned_port = listener.local_addr().unwrap().port();
        let listen_ip = format!("{}:{}", BASE_IP, assigned_port);
        let process = std::process::Command::new(exe_path)
            .args(["--listen-ip", &listen_ip])
            .spawn()?;
        Ok(Self { process, listen_ip })
    }

    pub fn listen_ip(&self) -> &str {
        &self.listen_ip
    }

    pub async fn send_ping(&self) -> String {
        let res = reqwest::Client::new()
            .get(format!("http://{}/ping", self.listen_ip()))
            .send()
            .await
            .expect("Error while sending request");
        res.text().await.expect("Error while parsing output")
    }

    pub async fn send_health(&self) -> Health {
        let res = reqwest::Client::new()
            .get(format!("http://{}/health", self.listen_ip()))
            .send()
            .await
            .expect("Error while sending request");
        res.json::<Health>()
            .await
            .expect("Error while parsing output")
    }

    pub async fn send_overview(&self) -> Overview {
        let res = reqwest::Client::new()
            .get(format!("http://{}/v1/overview", self.listen_ip()))
            .send()
            .await
            .expect("Error while sending request");
        res.json::<Overview>()
            .await
            .expect("Error while parsing output")
    }

    pub async fn send_query(&self, query: &str) -> Result<SqlQueryOutput, ParseErrorDTO> {
        let res = reqwest::Client::new()
            .post(format!("http://{}/v1/query", self.listen_ip()))
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
}

impl Drop for ServerInstance {
    fn drop(&mut self) {
        self.process.kill().expect("Couldn't kill server")
    }
}
