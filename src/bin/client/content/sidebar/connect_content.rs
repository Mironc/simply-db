use iced::{
    Alignment, Element, Length,
    widget::{self, pane_grid},
};

use crate::{
    Message,
    content::style::{MIN_SECTION_SIZE, button_style, text_input_style, text_style},
    global_data::GlobalData,
};
#[derive(Debug, Clone)]
pub struct ConnectContent {
    input_field: String,
    urls: Vec<String>,
}

impl ConnectContent {
    pub fn new() -> Self {
        Self {
            input_field: String::new(),
            urls: Vec::new(),
        }
    }
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        match message {
            Message::UrlFieldChanged(field) => self.input_field = field.clone(),
            Message::UrlSubmit(url) => {
                self.urls.push(url.to_owned());
                self.input_field = String::new();
            }
            Message::ConnectChoiceButton(url) => log::info!("{} db was chosen", url),
            _ => (),
        };
        iced::Task::none()
    }
    pub fn view<'a>(
        &'a self,
        _global_data: &GlobalData,
        is_expanded: bool,
        pane_id: pane_grid::Pane,
    ) -> Element<'a, Message> {
        let button = widget::button(
            widget::text!("DB connection")
                .align_x(Alignment::Center)
                .style(|_th| text_style()),
        )
        .on_press(Message::ToggleSidebar(pane_id))
        .style(|_th, status| button_style(status))
        .height(MIN_SECTION_SIZE)
        .width(Length::Fill)
        .clip(true);

        let content = if is_expanded {
            let urls = widget::column(self.urls.iter().map(|url| {
                widget::button(widget::text(url).style(|_th| text_style()))
                    .padding(5)
                    .on_press(Message::ConnectChoiceButton(url.clone()))
                    .style(|_th, status| button_style(status))
                    .into()
            }));
            widget::container(widget::column!(
                widget::text_input("type db url", &self.input_field)
                    .on_input(Message::UrlFieldChanged)
                    .on_submit(Message::UrlSubmit(self.input_field.clone()))
                    .style(|_th, status| text_input_style(status))
                    .width(300)
                    .align_x(Alignment::Center),
                urls
            ))
            .padding(7)
        } else {
            widget::container(widget::space())
        };
        widget::container(widget::column!(button, content))
            .width(Length::Fill)
            .clip(true)
            .into()
    }
}
