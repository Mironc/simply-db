use std::collections::HashMap;

use simply_db::{
    common_types::{DataValue, Schema},
    row::Row,
};

use crate::{
    AsyncMessage, Message,
    requests::{self, Overview},
};

#[derive(Debug, Clone, Default)]
pub struct GlobalData {
    chosen_url: Option<String>,
    chosen_table: Option<String>,
    fetched_rows: Option<Vec<Vec<DataValue>>>,
    fetched_overview: Option<Overview>,
}

impl GlobalData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chosen_url(&self) -> Option<&String> {
        self.chosen_url.as_ref()
    }

    pub fn chosen_table(&self) -> Option<&String> {
        self.chosen_table.as_ref()
    }

    pub fn fetched_rows(&self) -> Option<&Vec<Vec<DataValue>>> {
        self.fetched_rows.as_ref()
    }

    pub fn fetched_overview(&self) -> Option<&Overview> {
        self.fetched_overview.as_ref()
    }
    pub fn update(&mut self, message: &Message) -> iced::Task<Message> {
        match message {
            Message::AsyncMessage(mess) => match mess {
                AsyncMessage::OverviewResult(overview) => match overview {
                    Ok(res) => {
                        self.fetched_overview = Some(res.clone());
                    }
                    Err(e) => {
                        log::error!("{:?}", e);
                    }
                },
                AsyncMessage::FetchTableResult(items) => match items {
                    Ok(items) => self.fetched_rows = Some(items.clone()),
                    Err(e) => log::error!("{:?}", e),
                },
                AsyncMessage::QueryResult(result) => match result {
                    Ok(_) => (),
                    Err(e) => log::error!("{:?}", e),
                },
                _ => (),
            },
            Message::ConnectChoiceButton(url) => self.chosen_url = Some(url.clone()),
            Message::TableChoiceButton(table) => self.chosen_table = Some(table.clone()),
            Message::Update => {
                let mut task = iced::Task::none();
                if let Some(url) = &self.chosen_url {
                    let url = url.clone();
                    let task_1 =
                        iced::Task::perform(async {}, |()| Message::ConnectChoiceButton(url));
                    task = task.chain(task_1);
                };
                if let Some(table) = &self.chosen_table {
                    let table = table.clone();
                    let task_1 =
                        iced::Task::perform(async {}, |()| Message::TableChoiceButton(table));
                    task = task.chain(task_1);
                };
                return task;
            }
            _ => (),
        }
        iced::Task::none()
    }
}
