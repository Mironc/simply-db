use crate::{
    Message,
    content::style::{container_interactive_style, container_style, text_style},
    global_data::GlobalData,
};
use iced::{
    Alignment, Length,
    widget::{self, container},
};
#[derive(Debug, Clone, Default)]
pub struct TableViewer {}
impl TableViewer {
    pub fn view<'a>(&'a self, global_data: &'a GlobalData) -> widget::Container<'a, Message> {
        let content: iced::Element<_> = if let Some(rows) = global_data.fetched_rows()
            && let Some(table) = global_data.chosen_table()
            && let Some(schemas) = global_data.fetched_overview().map(|x| x.schemas())
            && let Some(schema) = schemas.get(table)
        {
            let header: widget::Container<'_, _> = container(widget::row(
                schema
                    .fields()
                    .keys()
                    .map(|field| {
                        widget::text(field.to_string())
                            .center()
                            .width(Length::Fill)
                            .style(|_th| text_style())
                            .into()
                    })
                    .collect::<Vec<iced::Element<'_, Message>>>(),
            ))
            .style(|_th| container_style())
            .width(Length::Fill);

            let rows = widget::column(
                rows.iter()
                    .map(|row| {
                        widget::row(
                            row.iter()
                                .map(|value| {
                                    widget::text(value.to_string())
                                        .center()
                                        .width(Length::Fill)
                                        .style(|_th| text_style())
                                        .into()
                                })
                                .collect::<Vec<iced::Element<'_, Message>>>(),
                        )
                        .into()
                    })
                    .collect::<Vec<iced::Element<'_, Message>>>(),
            );
            let scrollable = widget::scrollable(rows).width(Length::Fill);
            container(widget::column![header, scrollable])
                .style(|_th| container_interactive_style())
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(7)
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
