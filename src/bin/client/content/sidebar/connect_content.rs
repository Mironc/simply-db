use iced::{
    Alignment, Element, Length,
    widget::{self, pane_grid},
};

use crate::{
    Message,
    content::style::{
        MIN_SECTION_SIZE, button_style, container_style, text_input_style, text_style,
    },
    global_data::GlobalData,
};
#[derive(Debug, Clone)]
pub struct ConnectContent {
    input_field: String,
}

impl ConnectContent {
    pub fn new() -> Self {
        Self {
            input_field: String::new(),
        }
    }
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        match message {
            Message::UrlFieldChanged(field) => self.input_field = field.clone(),
            Message::UrlSubmit(_) => {
                self.input_field = String::new();
            }
            _ => (),
        };
        iced::Task::none()
    }
    pub fn view<'a>(
        &'a self,
        global_data: &GlobalData,
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
            let urls = widget::column(global_data.databases().iter().map(|(url, available)| {
                let connect_button =
                    widget::button(widget::text(url.clone()).style(|_th| text_style()))
                        .padding(5)
                        .on_press(Message::ConnectChoiceButton(url.clone()))
                        .style(|_th, status| button_style(status));

                let availability_indicator = widget::container(
                    widget::text(if *available { "A" } else { "O" })
                        .color(if *available {
                            iced::color!(0, 255, 0)
                        } else {
                            iced::color!(255, 0, 0)
                        })
                        .center(),
                )
                .style(|_th| container_style().border(iced::Border::default().width(0)))
                .padding(10)
                .width(Length::Shrink)
                .align_x(Alignment::End);

                let indicator_with_tooltip = widget::tooltip(
                    availability_indicator,
                    widget::container(widget::text(if *available {
                        "Server is available"
                    } else {
                        "Server is not available"
                    })),
                    widget::tooltip::Position::FollowCursor,
                );

                let delete_button = widget::container(
                    widget::button(
                        widget::text("X").center().color(iced::color!(255, 0, 0)), // RGB RED,
                    )
                    .style(|_th, status| button_style(status))
                    .on_press(Message::RemoveUrl(url.clone())),
                )
                .width(Length::Shrink)
                .align_x(Alignment::End);

                widget::row![
                    connect_button,
                    widget::space().width(Length::Fill),
                    indicator_with_tooltip,
                    delete_button
                ]
                .align_y(Alignment::Center)
                .into()
            }))
            .spacing(2);
            widget::container(
                widget::column!(
                    widget::text_input("type db url", &self.input_field)
                        .on_input(Message::UrlFieldChanged)
                        .on_submit(Message::UrlSubmit(self.input_field.clone()))
                        .style(|_th, status| text_input_style(status))
                        .width(300)
                        .align_x(Alignment::Center),
                    urls
                )
                .spacing(4),
            )
        } else {
            widget::container(widget::space())
        };
        widget::container(widget::column!(button, content).spacing(4))
            .padding(7)
            .width(Length::Fill)
            .clip(true)
            .into()
    }
}
