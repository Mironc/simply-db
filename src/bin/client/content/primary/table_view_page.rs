use iced::{
    Alignment, Length,
    widget::{self, container},
};
use storage::common_types::DataValue;

use crate::{
    AsyncMessage, Message,
    content::style::{container_interactive_style, container_style, text_style},
    global_data::GlobalData,
    requests,
};
#[derive(Debug, Clone, Default)]
pub struct TableViewer {}
impl TableViewer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self, global_data: &GlobalData, message: &Message) -> iced::Task<Message> {
        match message {
            Message::TableChoiceButton(table) => {
                if let Some(url) = global_data.chosen_url() {
                    iced::Task::perform(
                        requests::fetch_rows(url.clone(), table.clone()),
                        AsyncMessage::FetchTableResult,
                    )
                    .map(Message::AsyncMessage)
                } else {
                    iced::Task::none()
                }
            }
            _ => iced::Task::none(),
        }
    }
    pub fn view<'a>(&'a self, global_data: &'a GlobalData) -> widget::Container<'a, Message> {
        println!(
            "{:?} {:?} {:?}",
            global_data.fetched_rows(),
            global_data.chosen_table(),
            global_data.fetched_overview()
        );
        let content: iced::Element<_> = if let Some(rows) = global_data.fetched_rows()
            && let Some(table) = global_data.chosen_table()
            && let Some(schemas) = global_data.fetched_overview().map(|x| x.schemas())
            && let Some(schema) = schemas.get(table)
        {
            let mut columns = Vec::new();
            for (i, field) in schema.fields().iter().enumerate() {
                columns.push(widget::table::column(
                    widget::text(&field.0).style(|_th| text_style()),
                    move |row: &Vec<DataValue>| {
                        widget::text(row[i].to_string()).wrapping(widget::text::Wrapping::Glyph)
                    },
                ));
            }
            container(widget::table(columns, rows))
                .style(|_th| container_interactive_style())
                .padding(10)
                .into()
        } else {
            widget::space().into()
        };
        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .padding(7)
            .style(move |_th| container_style())
    }
}
