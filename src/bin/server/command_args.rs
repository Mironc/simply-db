use clap::Parser;
/// SimplyDB server executable
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct CommandArgs {
    /// Ip where database will be listening
    #[arg(short, long)]
    listen_ip: Option<String>,
}

impl CommandArgs {
    pub fn listen_ip(&self) -> &Option<String> {
        &self.listen_ip
    }
}
