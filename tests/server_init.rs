use tokio::net::TcpListener;

pub const BASE_IP: &'static str = "127.0.0.1";

pub async fn start_server() -> (std::io::Result<std::process::Child>, u16) {
    let exe_path = env!("CARGO_BIN_EXE_server");
    let listener = TcpListener::bind(format!("{}:0", BASE_IP)).await.unwrap(); //
    let assigned_port = listener.local_addr().unwrap().port();
    (
        std::process::Command::new(exe_path)
            .arg("--listen-ip")
            .arg(format!("{}:{}", BASE_IP, assigned_port))
            .spawn(),
        assigned_port,
    )
}
