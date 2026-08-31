use net::objects::Overview;
use storage::common_types::DataValue;

use crate::{AsyncMessage, Message, requests};

#[derive(Debug, Clone, Default)]
pub struct GlobalData {
    // database urls and last ping result
    databases: Vec<(String, bool)>,
    chosen_url: Option<String>,
    chosen_table: Option<String>,
    fetched_rows: Option<Vec<Vec<DataValue>>>,
    fetched_overview: Option<Overview>,
}

impl GlobalData {
    pub fn new() -> Self {
        Self::default()
    }
    fn fetch_overview(&self) -> iced::Task<Message> {
        if let Some(url) = &self.chosen_url {
            iced::Task::perform(
                requests::fetch_overview(url.clone()),
                AsyncMessage::OverviewResult,
            )
            .map(Message::AsyncMessage)
        } else {
            iced::Task::none()
        }
    }
    fn fetch_ping(&self) -> iced::Task<Message> {
        let mut futures = Vec::new();

        for (db, _) in self.databases.iter() {
            futures.push(requests::ping(db.clone()));
        }
        iced::Task::perform(
            iced::futures::future::join_all(futures),
            AsyncMessage::PingResult,
        )
        .map(Message::AsyncMessage)
    }
    fn fetch_rows(&self) -> iced::Task<Message> {
        if let (Some(url), Some(table)) = (&self.chosen_url, &self.chosen_table) {
            iced::Task::perform(
                requests::fetch_rows(url.clone(), table.clone()),
                AsyncMessage::FetchTableResult,
            )
            .map(Message::AsyncMessage)
        } else {
            iced::Task::none()
        }
    }
    fn fetch_query(&self, query: String) -> iced::Task<Message> {
        if let Some(url) = &self.chosen_url {
            iced::Task::perform(
                requests::send_query(url.clone(), query),
                AsyncMessage::QueryResult,
            )
            .map(Message::AsyncMessage)
        } else {
            iced::Task::none()
        }
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
                    Err(e) => log::error!("while fetching table: {:?}", e),
                },
                AsyncMessage::QueryResult(result) => match result {
                    Ok(_) => (),
                    Err(e) => log::error!("while sending query: {:?}", e),
                },
                AsyncMessage::PingResult(items) => {
                    for i in 0..self.databases.len() {
                        self.databases[i].1 = if let Ok(_) = items[i] { true } else { false };
                    }
                }
            },
            Message::ConnectChoiceButton(url) => {
                self.chosen_table = None;
                self.fetched_rows = None;
                self.fetched_overview = None;
                self.chosen_url = Some(url.clone());
                return self.fetch_overview();
            }
            Message::TableChoiceButton(table) => {
                self.fetched_rows = None;
                self.chosen_table = Some(table.clone());
                return self.fetch_rows();
            }
            Message::QuerySubmit(query) => return self.fetch_query(query.clone()),
            Message::UrlSubmit(url) => {
                self.databases.push((url.clone(), false));
            }
            Message::RemoveUrl(url) => {
                if let Some(pos) = self.databases.iter().position(|(db_url, _)| db_url == url) {
                    self.databases.remove(pos);
                } else {
                    log::error!("Url {} not found", url)
                }
            }
            Message::Update => {
                let mut tasks = Vec::new();
                tasks.push(self.fetch_ping());
                tasks.push(self.fetch_rows());
                tasks.push(self.fetch_overview());

                return iced::Task::batch(tasks);
            }
            _ => (),
        }
        iced::Task::none()
    }

    pub fn databases(&self) -> &[(String, bool)] {
        &self.databases
    }
}
