use iced::{
    Length,
    widget::{self, container},
};

use crate::{
    AsyncMessage, Message,
    content::style::{container_style, text_input_style},
    global_data::{self, GlobalData},
    requests::send_query,
};

#[derive(Debug, Clone, Default)]
pub struct QueryPage {
    query_string: String,
}
impl QueryPage {
    pub fn update(&mut self, global_data: &GlobalData, message: &Message) -> iced::Task<Message> {
        match message {
            Message::QueryFieldChanged(change) => self.query_string = change.clone(),
            Message::QuerySubmit(query) => {
                self.query_string = String::new();
                if let Some(url) = global_data.chosen_url() {
                    let task = iced::Task::perform(
                        send_query(url.clone(), query.clone()),
                        AsyncMessage::QueryResult,
                    )
                    .map(|x| Message::AsyncMessage(x));
                    return task;
                }
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
            .style(|_th| container_style())
    }
}
