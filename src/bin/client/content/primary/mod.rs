use iced::{
    Length,
    widget::{self, container},
};

use crate::{
    Message,
    content::{
        primary::{query_page::QueryPage, table_view_page::TableViewer},
        style::{button_style, container_interactive_style, container_style, text_style},
    },
    global_data::GlobalData,
};

mod query_page;
mod table_view_page;
#[derive(Debug, Clone, Copy)]
pub enum PrimaryPage {
    TableView,
    QueryPage,
}

impl Default for PrimaryPage {
    fn default() -> Self {
        PrimaryPage::TableView
    }
}
#[derive(Debug, Clone, Default)]
pub struct PrimaryContent {
    table_view: TableViewer,
    query_page: QueryPage,
    page: PrimaryPage,
}

impl PrimaryContent {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update(&mut self, message: &Message, global_data: &GlobalData) -> iced::Task<Message> {
        match message {
            Message::PrimarySwitchPage(primary_page) => self.page = *primary_page,
            _ => (),
        }
        let t1 = self.query_page.update(global_data, message);
        let t2 = self.table_view.update(global_data, message);
        t1.chain(t2)
    }
    pub fn view<'a>(&'a self, global_data: &'a GlobalData) -> iced::widget::Container<'a, Message> {
        let table_view_header =
            widget::button(widget::text("Table view").style(|_th| text_style()))
                .width(Length::FillPortion(1))
                .on_press(Message::PrimarySwitchPage(PrimaryPage::TableView))
                .style(|_th, status| button_style(status));
        let query_page_header = widget::button(widget::text("Query").style(|_th| text_style()))
            .width(Length::FillPortion(1))
            .on_press(Message::PrimarySwitchPage(PrimaryPage::QueryPage))
            .style(|_th, status| button_style(status));

        let header = widget::row![table_view_header, query_page_header];
        let page_content = match &self.page {
            PrimaryPage::TableView => self.table_view.view(global_data),
            PrimaryPage::QueryPage => self.query_page.view(),
        };
        container(widget::column![header, page_content])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_th| container_style())
            .into()
    }
}
