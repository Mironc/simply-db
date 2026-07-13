mod requests;
mod server_init;
#[cfg(test)]
mod tests {
    use net::objects::DatabaseState;
    use storage::scalar;

    use crate::{
        requests::{send_health, send_ping, send_query},
        server_init::{BASE_IP, start_server},
    };

    #[tokio::test]
    async fn test_setup() {
        let (server_app, server_port) = start_server().await;
        let mut server_app = server_app.expect("Couldn't start server");
        let server_listen_ip = format!("{}:{}", BASE_IP, server_port);
        let ping_text = send_ping(&server_listen_ip).await;
        assert_eq!(ping_text, "pong");

        let health = send_health(&server_listen_ip).await;
        assert_eq!(health.state(), DatabaseState::Healthy);

        server_app.kill().expect("Couldn't kill process")
    }
    #[tokio::test]
    async fn basic_queries() {
        let (server_app, server_port) = start_server().await;
        let mut server_app = server_app.expect("Couldn't start server");
        let server_listen_ip = format!("{}:{}", BASE_IP, server_port);
        send_query(&server_listen_ip, "CREATE TABLE users (id INT, name TEXT)")
            .await
            .unwrap();

        send_query(
            &server_listen_ip,
            "INSERT INTO users (id, name) VALUES (0, 'Steve'), (1,'Alice')",
        )
        .await
        .unwrap();

        let output = send_query(&server_listen_ip, "SELECT * FROM users")
            .await
            .unwrap();
        match output.output()[0].as_ref().unwrap() {
            query::QueryOutput::Rows(items) => {
                assert_eq!(
                    items,
                    &vec![
                        vec![scalar!(Int(0)), scalar!(Text("Steve".to_owned()))],
                        vec![scalar!(Int(1)), scalar!(Text("Alice".to_owned()))]
                    ]
                );
            }
            query::QueryOutput::Nothing => panic!("Expected rows"),
        }

        server_app.kill().expect("Couldn't kill process")
    }
}
