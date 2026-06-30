use iced::{
    Length,
    widget::{self, pane_grid},
};

use crate::{
    Message,
    content::{
        header::Header,
        primary::PrimaryContent,
        sidebar::SidebarContent,
        style::{container_style, container_whole_style, text_style},
    },
    global_data::{self, GlobalData},
};
pub mod header;
pub mod primary;
pub mod sidebar;
pub mod style;

#[derive(Debug, Clone)]
pub enum ContentPart {
    Sidebar(SidebarContent),
    Primary(PrimaryContent),
}
impl ContentPart {
    pub fn title(&self) -> &str {
        match self {
            ContentPart::Sidebar(_) => "Sidebar",
            ContentPart::Primary(_) => "Primary",
        }
    }
    pub fn update(&mut self, message: &Message, global_data: &GlobalData) -> iced::Task<Message> {
        match self {
            ContentPart::Sidebar(sidebar) => sidebar.update(&message),
            ContentPart::Primary(primary) => primary.update(&message, global_data),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Content {
    grid: pane_grid::State<ContentPart>,
    header: Header,
}

impl Content {
    pub fn new() -> Self {
        let (mut grid, sidebar_pane) =
            pane_grid::State::new(ContentPart::Sidebar(SidebarContent::new()));
        let (_, _) = grid
            .split(
                pane_grid::Axis::Vertical,
                sidebar_pane,
                ContentPart::Primary(PrimaryContent::new()),
            )
            .unwrap();
        let header = Header {};
        Self { grid, header }
    }
    pub fn update(&mut self, message: Message, global_data: &GlobalData) -> iced::Task<Message> {
        let mut tasks = iced::Task::none();
        for pane in self.grid.iter_mut() {
            tasks = tasks.chain(pane.1.update(&message, global_data));
        }
        match message {
            Message::MainResize(resize_event) => {
                self.grid.resize(resize_event.split, resize_event.ratio);
                tasks
            }
            _ => tasks,
        }
    }
    pub fn view<'a>(&'a self, global_data: &'a GlobalData) -> iced::Element<'a, Message> {
        let header_view = self.header.view();
        let grid_view = widget::PaneGrid::new(
            &self.grid,
            |_id, content, _is_maximized| -> pane_grid::Content<'_, Message> {
                let title = pane_grid::TitleBar::new(
                    widget::text(content.title())
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .style(|_th| text_style())
                        .center(),
                )
                .padding(7)
                .style(|_th| container_style());
                pane_grid::Content::new(match content {
                    ContentPart::Sidebar(sidebar) => sidebar.view(global_data),
                    ContentPart::Primary(primary) => primary.view(global_data),
                })
                .title_bar(title)
            },
        )
        .on_resize(10.0, Message::MainResize)
        .min_size(125)
        .height(Length::Fill)
        .width(Length::Fill);
        widget::container(widget::column![header_view, grid_view])
            .padding(3)
            .style(|_th| container_whole_style())
            .into()
    }
}
