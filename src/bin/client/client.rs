use iced::widget::pane_grid;
use simply_db::common_types::DataValue;

use crate::{
    content::{Content, primary::PrimaryPage},
    global_data::GlobalData,
    requests::{FetchError, Overview},
};

mod content;
mod global_data;
mod requests;

fn init_logger() {
    use env_logger::fmt::style::AnsiColor;
    use env_logger::{Builder, Env};
    use std::io::Write;
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::new()
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => AnsiColor::BrightRed,
                log::Level::Warn => AnsiColor::Yellow,
                log::Level::Info => AnsiColor::Blue,
                log::Level::Debug => AnsiColor::Magenta,
                log::Level::Trace => AnsiColor::BrightGreen,
            };
            let default_level_style = AnsiColor::Black;
            writeln!(
                buf,
                "{}[{}]{}[{}] - {}",
                level_style.render_fg(),
                record.level(),
                default_level_style.render_fg(),
                record.target(),
                record.args()
            )
        })
        .parse_env(env)
        .init();
}
pub fn main() -> iced::Result {
    #[cfg(target_os = "windows")]
    {
        if std::env::var("WGPU_BACKEND").is_err() {
            // IDK why but wgpu with dx12 is crashing
            unsafe { std::env::set_var("WGPU_BACKEND", "dx11") };
        }
    }
    init_logger();
    log::info!("Running app");
    iced::application(app_boot, app_update, app_view)
        .window_size((500, 500))
        .title("simply_db Client")
        .run()
}
#[derive(Debug, Clone)]
pub enum AsyncMessage {
    Ping,
    PingResult(Result<(), FetchError>),
    Overview,
    OverviewResult(Result<Overview, FetchError>),
    FetchTable,
    FetchTableResult(Result<Vec<Vec<DataValue>>, FetchError>),
    SendQuery,
    QueryResult(Result<(), FetchError>),
}
#[derive(Debug, Clone)]
pub enum Message {
    MainResize(pane_grid::ResizeEvent),
    SidebarResize(pane_grid::ResizeEvent),
    ConnectChoiceButton(String),
    UrlFieldChanged(String),
    QueryFieldChanged(String),
    QuerySubmit(String),
    PrimarySwitchPage(PrimaryPage),
    ToggleSidebar(pane_grid::Pane),
    UrlSubmit(String),
    AsyncMessage(AsyncMessage),
    Update,
    TableChoiceButton(String),
}
#[derive(Debug, Clone)]
pub struct AppState {
    content: Content,
    global_data: GlobalData,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            content: Content::new(),
            global_data: GlobalData::new(),
        }
    }
}
fn app_boot() -> AppState {
    AppState::new()
}
fn app_update(state: &mut AppState, message: Message) -> iced::Task<Message> {
    let task1 = state.global_data.update(&message);
    let task2 = state.content.update(message, &state.global_data);
    task1.chain(task2)
}
fn app_view<'a>(state: &AppState) -> iced::Element<'_, Message> {
    state.content.view(&state.global_data)
}
