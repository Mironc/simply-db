#[cfg(test)]
mod tests {
    use simply_db::net::objects::DatabaseState;
    use storage::{common_types::ScalarType, scalar, schema::FieldType};

    use crate::server_tests::server_init::ServerInstance;

    #[tokio::test]
    async fn test_setup() {
        let instance = ServerInstance::new().await.expect("Couldn't start server");

        let ping_text = instance.send_ping().await;
        assert_eq!(ping_text, "pong");

        let health = instance.send_health().await;
        assert_eq!(health.state(), DatabaseState::Healthy);
    }
    #[tokio::test]
    async fn basic_queries() {
        let instance = ServerInstance::new().await.expect("Couldn't start server");
        instance
            .send_query("CREATE TABLE users (id INT, name TEXT)")
            .await
            .unwrap();

        instance
            .send_query("INSERT INTO users (id, name) VALUES (0, 'Steve'), (1,'Alice')")
            .await
            .unwrap();

        let overview = instance.send_overview().await;
        let fields = overview.schemas().get("users").unwrap().fields();
        assert_eq!(
            fields.get("id").unwrap(),
            &FieldType::new(ScalarType::Int, vec![])
        );
        assert_eq!(
            fields.get("name").unwrap(),
            &FieldType::new(ScalarType::Text, vec![])
        );

        let output = instance.send_query("SELECT * FROM users").await.unwrap();
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
    }
}
