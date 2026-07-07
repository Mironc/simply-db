use iced::{
    Alignment, Length,
    widget::{self, container, pane_grid},
};
pub mod connect_content;
pub mod overview_content;
use crate::{
    Message,
    content::style::{MIN_SECTION_SIZE, container_style},
    global_data::GlobalData,
};
use connect_content::*;
use overview_content::*;

#[derive(Debug, Clone)]
pub enum SidebarPart {
    Connection(ConnectContent),
    Overview(OverviewContent),
}
#[derive(Debug, Clone)]
pub struct SidebarSection {
    is_expanded: bool,
    part: SidebarPart,
}
impl SidebarSection {
    pub fn new(is_expanded: bool, part: SidebarPart) -> Self {
        Self { is_expanded, part }
    }
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        match &mut self.part {
            SidebarPart::Connection(connect_content) => connect_content.update(message),
            SidebarPart::Overview(overview_content) => overview_content.update(message),
        }
    }
    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }
}
#[derive(Debug, Clone)]
pub struct SidebarSplit {
    pane: pane_grid::Pane,
    other_pane: pane_grid::Pane,
    split: pane_grid::Split,
    ratio: f32,
}

impl SidebarSplit {
    pub fn new(
        pane: pane_grid::Pane,
        other_pane: pane_grid::Pane,
        split: pane_grid::Split,
        ratio: f32,
    ) -> Self {
        Self {
            split,
            ratio,
            pane,
            other_pane,
        }
    }
}
#[derive(Debug, Clone)]
pub struct SidebarContent {
    grid: pane_grid::State<SidebarSection>,
    splits: Vec<SidebarSplit>,
}

impl SidebarContent {
    pub fn new() -> Self {
        let (mut grid, sidebar_connect) = pane_grid::State::new(SidebarSection::new(
            true,
            SidebarPart::Connection(ConnectContent::new()),
        ));
        let (sidebar_overview, sidebar_split) = grid
            .split(
                pane_grid::Axis::Horizontal,
                sidebar_connect,
                SidebarSection::new(true, SidebarPart::Overview(OverviewContent::new())),
            )
            .unwrap();

        Self {
            grid,
            splits: vec![SidebarSplit::new(
                sidebar_connect,
                sidebar_overview,
                sidebar_split,
                0.5,
            )],
        }
    }
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        let mut tasks = iced::Task::none();
        for pane in self.grid.iter_mut() {
            tasks = tasks.chain(pane.1.update(message));
        }
        match message {
            Message::SidebarResize(resize_event) => {
                let split = self
                    .splits
                    .iter_mut()
                    .find(|x| x.split == resize_event.split)
                    .unwrap();
                if let Some(pane) = self.grid.get(split.pane) {
                    split.ratio = resize_event.ratio;
                    self.grid.resize(
                        resize_event.split,
                        if pane.is_expanded {
                            resize_event.ratio
                        } else {
                            split.ratio
                        },
                    );
                    tasks
                } else {
                    panic!("WTF")
                }
            }
            Message::ToggleSidebar(pane_id) => {
                if let Some(pane_state) = self.grid.get_mut(*pane_id) {
                    pane_state.is_expanded = !pane_state.is_expanded;
                    if let Some(split) = self
                        .splits
                        .iter()
                        .find(|x| &x.pane == pane_id || &x.other_pane == pane_id)
                    {
                        let is_expanded = pane_state.is_expanded;
                        let target_ratio = if is_expanded { split.ratio } else { 0.0 };
                        self.grid.resize(split.split, target_ratio);
                    }
                }
                tasks
            }
            _ => tasks,
        }
    }
    pub fn view<'a>(&'a self, global_data: &'a GlobalData) -> widget::Container<'a, Message> {
        container(
            widget::PaneGrid::new(
                &self.grid,
                |pane_id, content, _is_maximized| -> pane_grid::Content<'a, Message> {
                    let content = match &content.part {
                        SidebarPart::Connection(c_content) => {
                            c_content.view(global_data, content.is_expanded(), pane_id)
                        }
                        SidebarPart::Overview(o_content) => {
                            o_content.view(global_data, content.is_expanded, pane_id)
                        }
                    };
                    pane_grid::Content::new(content).style(move |_th| container_style())
                },
            )
            .on_resize(10.0, Message::SidebarResize)
            .min_size(MIN_SECTION_SIZE)
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .style(move |_th| container_style())
    }
}
