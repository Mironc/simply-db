use iced::{
    Alignment, Element, Length,
    widget::{self, pane_grid},
};

use crate::{
    AsyncMessage, Message,
    content::style::{MIN_SECTION_SIZE, button_style, text_style},
    global_data::GlobalData,
    requests::{self},
};

#[derive(Debug, Clone, Default)]
pub struct OverviewContent {}

impl OverviewContent {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        match message {
            Message::ConnectChoiceButton(url) => iced::Task::perform(
                requests::fetch_overview(url.clone()),
                AsyncMessage::OverviewResult,
            )
            .map(Message::AsyncMessage),
            _ => iced::Task::none(),
        }
    }
    pub fn view<'a>(
        &'a self,
        global_data: &'a GlobalData,
        is_expanded: bool,
        pane_id: pane_grid::Pane,
    ) -> Element<'a, Message> {
        let button = widget::button(
            widget::text!("DB overview")
                .align_x(Alignment::Center)
                .style(|_th| text_style()),
        )
        .on_press(Message::ToggleSidebar(pane_id))
        .style(|_th, status| button_style(status))
        .height(MIN_SECTION_SIZE)
        .width(Length::Fill)
        .clip(true);
        let content = if is_expanded {
            let mut content = widget::Column::new();
            if let Some(overview) = global_data.fetched_overview() {
                for x in overview.schemas().keys() {
                    content = content.push(
                        widget::button(widget::text(x).style(|_th| text_style()))
                            .on_press(Message::TableChoiceButton(x.clone()))
                            .style(|_th, status| button_style(status))
                            .clip(true),
                    );
                }
            }
            widget::container(content)
        } else {
            widget::container(widget::space())
        }
        .padding(7);
        widget::container(widget::column!(button, content))
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .into()
    }
}
