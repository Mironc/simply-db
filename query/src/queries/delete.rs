use std::ops::Deref;

use storage::{
    common_types::{DataValue, ScalarValue},
    db::Database,
};

use crate::{
    context::Context,
    expr::{Expr, ExprError},
};

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteError {
    UnknownTable(String),
    ExprErr(ExprError),
    BadExpr,
}
impl From<ExprError> for DeleteError {
    fn from(v: ExprError) -> Self {
        Self::ExprErr(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteQuery {
    DropTable(DropTable),
    DeleteRows(DeleteRows),
    TruncateTable(TruncateTable),
}

impl DeleteQuery {
    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        match self {
            DeleteQuery::DropTable(drop_table) => drop_table.execute(db),
            DeleteQuery::DeleteRows(delete_rows) => delete_rows.execute(db),
            DeleteQuery::TruncateTable(truncate_table) => truncate_table.execute(db),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DropTable {
    table: String,
}

impl DropTable {
    pub fn new(table: String) -> Self {
        Self { table }
    }
    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        if db.delete_table(&self.table).is_none() {
            return Err(DeleteError::UnknownTable(self.table.clone()));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteRows {
    table: String,
    expr: Expr,
}
impl DeleteRows {
    pub fn new(table: String, expr: Expr) -> Self {
        Self { table, expr }
    }
    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        let table = if let Some(table) = db.get_table(&self.table) {
            table
        } else {
            return Err(DeleteError::UnknownTable(self.table.clone()));
        };
        let mut rows = Vec::new();
        for (i, row) in table.rows().iter().enumerate() {
            let context = Context::new(row.data(), table.schema());
            let val = self.expr.execute(&context)?;

            match val.deref() {
                DataValue::Null => continue,
                DataValue::Scalar(ScalarValue::Bool(b)) => {
                    if *b {
                        rows.push(i);
                    } else {
                        continue;
                    }
                }
                _ => return Err(DeleteError::BadExpr),
            }
        }
        for (i, row_idx) in rows.iter().enumerate() {
            table.rows_mut().remove(row_idx - i);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TruncateTable {
    table: String,
}

impl TruncateTable {
    pub fn new(table: String) -> Self {
        Self { table }
    }

    pub fn execute(&self, db: &Database) -> Result<(), DeleteError> {
        let table = if let Some(table) = db.get_table(&self.table) {
            table
        } else {
            return Err(DeleteError::UnknownTable(self.table.clone()));
        };
        table.rows_mut().clear();
        Ok(())
    }
}
#[cfg(test)]
mod tests {

    use storage::{
        common_types::ScalarType,
        db::Database,
        row::Row,
        scalar,
        schema::{FieldType, Schema},
        table::Table,
    };
    use structures::VecMap;

    use crate::{
        expr::{ComparisonOp, Expr, LiteralValue},
        queries::delete::{DeleteError, DeleteRows, DropTable, TruncateTable},
    };

    pub fn init_db() -> Database {
        let db = Database::new();
        // Create first row
        let mut data = Vec::new();
        data.push(scalar!(Int(30)));
        data.push(scalar!(Text("Alice".to_owned())));
        data.push(scalar!(Bool(true)));
        let row1 = Row::new(data);

        // Create second row
        let mut data = Vec::new();
        data.push(scalar!(Int(25)));
        data.push(scalar!(Text("Bob".to_owned())));
        data.push(scalar!(Bool(false)));
        let row2 = Row::new(data);

        let mut field_types = VecMap::new();
        field_types.insert("age".to_string(), FieldType::new(ScalarType::Int, vec![]));
        field_types.insert("name".to_string(), FieldType::new(ScalarType::Text, vec![]));
        field_types.insert(
            "is_active".to_string(),
            FieldType::new(ScalarType::Bool, vec![]),
        );
        let schema = Schema::new(field_types);
        // Create table
        let table = Table::new(schema);

        // Insert into database
        db.insert_table("test_table".to_string(), table).unwrap();
        let table = db.get_table("test_table").unwrap();
        table
            .insert_row(
                &vec!["age".to_owned(), "name".to_owned(), "is_active".to_owned()],
                row1,
            )
            .unwrap();
        table
            .insert_row(
                &vec!["age".to_owned(), "name".to_owned(), "is_active".to_owned()],
                row2,
            )
            .unwrap();

        db
    }
    #[test]
    fn delete_success() {
        let db = init_db();
        let expr = Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("is_active".to_owned()),
            Expr::Literal(LiteralValue::Bool(false)),
        )));
        let delete = DeleteRows::new("test_table".to_owned(), expr);
        delete.execute(&db).unwrap();
        assert_eq!(
            db.get_table("test_table").map(|x| x.rows().len()),
            Some(1),
            "Row didn't get deleted"
        );
    }
    #[test]
    fn delete_bad_expr() {
        let db = init_db();
        let expr = Expr::Literal(LiteralValue::Text("BadExpr".to_owned()));
        let delete = DeleteRows::new("test_table".to_owned(), expr);
        assert_eq!(delete.execute(&db), Err(DeleteError::BadExpr));
    }
    #[test]
    fn delete_unknown_table() {
        let db = init_db();
        let expr = Expr::Comparison(Box::new(ComparisonOp::Eq(
            Expr::Field("is_active".to_owned()),
            Expr::Literal(LiteralValue::Bool(false)),
        )));
        let delete = DeleteRows::new("unknown_table".to_owned(), expr);
        assert_eq!(
            delete.execute(&db),
            Err(DeleteError::UnknownTable("unknown_table".to_owned()))
        );
    }
    #[test]
    fn truncate_success() {
        let db = init_db();
        let trunc = TruncateTable::new("test_table".to_owned());
        trunc.execute(&db).unwrap();
        assert_eq!(
            db.get_table("test_table").map(|x| x.rows().len()),
            Some(0),
            "Rows didn't get deleted"
        );
    }
    #[test]
    fn truncate_unknown_table() {
        let db = init_db();
        let trunc = TruncateTable::new("unknown_table".to_owned());
        assert_eq!(
            trunc.execute(&db),
            Err(DeleteError::UnknownTable("unknown_table".to_owned())),
        );
    }

    #[test]
    fn drop_success() {
        let db = init_db();
        let drop = DropTable::new("test_table".to_owned());
        drop.execute(&db).unwrap();
        assert!(!db.has_table("test_table"));
    }
    #[test]
    fn drop_unknown_table() {
        let db = init_db();
        let drop = DropTable::new("unknown_table".to_owned());

        assert_eq!(
            drop.execute(&db),
            Err(DeleteError::UnknownTable("unknown_table".to_owned()))
        );
    }
}
