use iced::{
    Alignment, Element, Length,
    widget::{self, container},
};

use crate::{
    Message,
    content::style::{button_style, container_style, text_style},
};

#[derive(Debug, Clone)]
pub struct Header {}

impl Header {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self) {}
    pub fn view(&self) -> Element<'_, Message> {
        container(
            widget::button(
                widget::text!("Upd")
                    .align_x(Alignment::Center)
                    .style(|_th| text_style()),
            )
            .on_press(Message::Update)
            .style(|_th, status| button_style(status)),
        )
        .style(|_th| container_style())
        .width(Length::Fill)
        .align_x(Alignment::End)
        .into()
    }
}
