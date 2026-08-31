use iced::{
    Length,
    widget::{self, container},
};

use crate::{
    Message,
    content::style::{container_style, text_input_style},
};

#[derive(Debug, Clone, Default)]
pub struct QueryPage {
    query_string: String,
}
impl QueryPage {
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        match message {
            Message::QueryFieldChanged(change) => self.query_string = change.clone(),
            Message::QuerySubmit(_) => {
                self.query_string = String::new();
            }
            _ => (),
        }
        iced::Task::none()
    }
    pub fn view(&self) -> widget::Container<'_, Message> {
        let text_input = widget::text_input("Write SQL query", &self.query_string)
            .style(|_th, status| text_input_style(status))
            .on_input(Message::QueryFieldChanged)
            .on_submit(Message::QuerySubmit(self.query_string.clone()));
        container(text_input)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(7)
            .style(|_th| container_style())
    }
}
