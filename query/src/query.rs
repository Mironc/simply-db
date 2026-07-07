use storage::{common_types::DataValue, db::Database};

use crate::queries::{
    create_table::{CreateTable, CreateTableError},
    delete::{DeleteError, DeleteQuery},
    insert::{InsertError, InsertQuery},
    select::{SelectError, SelectQuery},
    update::{UpdateError, UpdateQuery},
};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum QueryError {
    FilterError(SelectError),
    InsertError(InsertError),
    CreateTableError(CreateTableError),
    UpdateError(UpdateError),
    DeleteError(DeleteError),
}

impl From<DeleteError> for QueryError {
    fn from(v: DeleteError) -> Self {
        Self::DeleteError(v)
    }
}

impl From<UpdateError> for QueryError {
    fn from(v: UpdateError) -> Self {
        Self::UpdateError(v)
    }
}
impl From<SelectError> for QueryError {
    fn from(value: SelectError) -> Self {
        Self::FilterError(value)
    }
}
impl From<InsertError> for QueryError {
    fn from(value: InsertError) -> Self {
        Self::InsertError(value)
    }
}
impl From<CreateTableError> for QueryError {
    fn from(value: CreateTableError) -> Self {
        Self::CreateTableError(value)
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum QueryOutput {
    Rows(Vec<Vec<DataValue>>),
    Nothing,
}
impl From<Vec<Vec<DataValue>>> for QueryOutput {
    fn from(v: Vec<Vec<DataValue>>) -> Self {
        Self::Rows(v)
    }
}
impl From<()> for QueryOutput {
    fn from(_: ()) -> Self {
        Self::Nothing
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Select(SelectQuery),
    Insert(InsertQuery),
    CreateTable(CreateTable),
    Update(UpdateQuery),
    Delete(DeleteQuery),
}
impl Query {
    pub fn apply(&self, db: &Database) -> Result<QueryOutput, QueryError> {
        Ok(match self {
            Query::Select(filter_query) => filter_query.execute(db)?.into(),
            Query::Insert(insert_query) => insert_query.execute(db)?.into(),
            Query::CreateTable(create_table) => create_table.execute(db)?.into(),
            Query::Update(update_query) => update_query.execute(db)?.into(),
            Query::Delete(delete_query) => delete_query.execute(db)?.into(),
        })
    }
}
#[derive(Debug, Clone)]
pub struct QueryRequest {
    queries: Vec<Query>,
}
impl QueryRequest {
    pub fn new(queries: Vec<Query>) -> Self {
        Self { queries }
    }
    pub fn execute(&self, db: &Database) -> Vec<Result<QueryOutput, QueryError>> {
        let mut results = Vec::new();
        for query in self.queries.iter() {
            let res = query.apply(db);
            results.push(res);
        }
        results
    }
}
