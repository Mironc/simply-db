use std::{borrow::Cow, collections::HashMap, ops::Deref};

use crate::{
    common_types::{DataType, DataValue, FieldType, ScalarType, Schema, SchemaValue},
    queries::{
        create_table::CreateTable,
        delete::{DeleteQuery, DeleteRows, DropTable, TruncateTable},
        insert::InsertQuery,
        select::{Projection, SelectQuery},
        update::UpdateQuery,
    },
    sql::{
        expr::Expr,
        parser::{
            common::{ParseError, ParseResult, TokenWalker, parse_field, parse_literal},
            expr::parse_expr,
            tokenizer::{Delimiter, TokenValue},
        },
        query::Query,
    },
};

use super::tokenizer::Sign;

pub fn parse_query(tokens: Vec<TokenValue>) -> ParseResult<Query> {
    let walker = TokenWalker::new(&tokens);
    match walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnknownInstruction)?
    {
        TokenValue::Keyword(Cow::Borrowed("INSERT")) => parse_insert_query(walker),
        TokenValue::Keyword(Cow::Borrowed("CREATE")) => parse_create_query(walker),
        TokenValue::Keyword(Cow::Borrowed("SELECT")) => parse_select_query(walker),
        TokenValue::Keyword(Cow::Borrowed("UPDATE")) => parse_update_query(walker),
        TokenValue::Keyword(Cow::Borrowed("DROP")) => parse_drop_query(walker),
        TokenValue::Keyword(Cow::Borrowed("TRUNCATE")) => parse_truncate_query(walker),
        TokenValue::Keyword(Cow::Borrowed("DELETE")) => parse_delete_query(walker),
        _ => Err(ParseError::UnknownInstruction),
    }
}
pub(super) fn parse_update_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("UPDATE".into()))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit".into(),
            given: table_name_token.to_string().into(),
        });
    }
    let table_name = table_name_token.to_string();

    walker.expect_next_token(&TokenValue::Keyword("SET".into()))?;
    let mut set_exprs = Vec::new();
    'outer: while let Some(token) = walker.peek_next_meaningful()
        && token != &TokenValue::Keyword("WHERE".into())
    {
        let field_name = parse_field(&mut walker)?;
        walker.expect_next_token(&TokenValue::Sign(Sign::Set))?;
        let mut clone = walker.clone();
        let mut next_token = walker.next_meaningful();
        while let Some(token) = next_token {
            if token == &TokenValue::Delimiter(Delimiter::Comma) {
                break;
            }
            if token == &TokenValue::Keyword("WHERE".into()) {
                set_exprs.push((field_name, parse_expr(&mut clone, walker.position())?));
                break 'outer;
            }
            next_token = walker.next_meaningful();
        }
        set_exprs.push((field_name, parse_expr(&mut clone, walker.position())?));
    }
    let filter_expr = if let Some(token) = walker.current_token()
        && token == &TokenValue::Keyword("WHERE".into())
    {
        if token != &TokenValue::Keyword("WHERE".into()) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "WHERE clause".into(),
                given: token.to_string().into(),
            });
        }
        let end = walker.tokens().len();
        Some(parse_expr(&mut walker, end)?)
    } else {
        None
    };
    let query = UpdateQuery::new(table_name, set_exprs, filter_expr);
    Ok(Query::Update(query))
}
pub(super) fn parse_select_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("SELECT".into()))?;

    let projection = if walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnexpectedEof)?
        == &TokenValue::Sign(Sign::Asterisk)
    {
        walker.skip_meaningful(1);
        walker.expect_next_token(&TokenValue::Keyword("FROM".into()))?;
        Projection::Row
    } else {
        let mut expressions = Vec::new();
        let mut open = 0;
        let mut walker_new = walker.clone_simple();
        let mut token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        while token != &TokenValue::Keyword("FROM".into()) {
            if token.to_string() == "(" {
                open += 1
            }
            if token.to_string() == ")" {
                if open == 0 {
                    return Err(ParseError::UnclosedBracket(')'));
                }
                open -= 1
            }
            if token.to_string() == "," && open == 0 {
                expressions.push(parse_expr(&mut walker_new, walker.position())?);
                walker_new = walker.clone_simple();
            }
            token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        }
        expressions.push(parse_expr(&mut walker_new, walker.position())?);
        Projection::Expr(expressions)
    };

    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit".into(),
            given: table_name_token.to_string().into(),
        });
    }
    let table_name = table_name_token.to_string();
    let filter_expr = if let Some(TokenValue::Keyword(k)) = walker.peek_next_meaningful()
        && k == "WHERE"
    {
        walker.skip(2);
        let end = walker.tokens().len();
        println!("WHERE {:?}", &walker.tokens()[walker.position()..end]);
        Some(parse_expr(&mut walker, end)?)
    } else {
        None
    };
    Ok(Query::Select(SelectQuery::new(
        table_name,
        projection,
        filter_expr,
    )))
}
/// Parses CREATE TABLE query
pub(super) fn parse_create_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("CREATE".into()))?;
    walker.expect_next_token(&TokenValue::Keyword("TABLE".into()))?;

    let mut if_not_exists = false;
    if let (
        Some(TokenValue::Keyword(Cow::Borrowed("IF"))),
        Some(TokenValue::Keyword(Cow::Borrowed("NOT"))),
        Some(TokenValue::Keyword(Cow::Borrowed("EXISTS"))),
    ) = (
        walker.peek_n_meaningful(1),
        walker.peek_n_meaningful(2),
        walker.peek_n_meaningful(3),
    ) {
        if_not_exists = true;
        walker.skip_meaningful(3).unwrap();
    }
    let table_name = {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        println!("{:?}", token);
        if token.is_ident() {
            if !token.starts_with_digit() {
                token.to_string()
            } else {
                return Err(ParseError::Other {
                    message: "Expected valid table name which not starts with digit".into(),
                });
            }
        } else {
            return Err(ParseError::Other {
                message: "Expected table name".into(),
            });
        }
    };
    let row_type = parse_create_fields(&mut walker)?;
    let create_table_query = CreateTable::new(table_name, row_type, if_not_exists);
    Ok(Query::CreateTable(create_table_query))
}
pub(super) fn parse_create_fields(walker: &mut TokenWalker) -> ParseResult<Schema> {
    let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if token != &TokenValue::Delimiter(Delimiter::RoundOpen) {
        return Err(ParseError::UnexpectedSymbol {
            expected: "(".into(),
            given: token.to_string().into(),
        });
    }
    let mut fields = Vec::new();
    loop {
        let field_parsed = parse_field_and_modifiers(walker)?;
        // TODO: FIELD MODIFIERS
        let field = FieldType::new(
            field_parsed.1,
            !field_parsed.2.contains(&FieldModifier::NotNull)
                && !field_parsed.2.contains(&FieldModifier::PrimaryKey)
                && !field_parsed.2.contains(&FieldModifier::AutoIncrement),
        );
        fields.push((field_parsed.0, field));
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if let TokenValue::Delimiter(Delimiter::RoundClose) = token {
            break;
        }
        if token != &TokenValue::Delimiter(Delimiter::Comma) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "COMMA or )".into(),
                given: token.to_string().into(),
            });
        }
    }

    Ok(Schema::new(fields))
}
pub(super) fn parse_field_and_modifiers(
    walker: &mut TokenWalker,
) -> ParseResult<(String, DataType, FieldModifiers)> {
    let field_name = parse_field(walker)?;
    let field_type = {
        let field_type_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if let Some(field_type) = ScalarType::from_str(&field_type_token.to_string()) {
            DataType::Scalar(field_type)
        } else {
            return Err(ParseError::Other {
                message: "Expected field type".into(),
            });
        }
    };
    let mut field_modifiers = Vec::new();
    while walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnexpectedEof)?
        .is_keyword()
    {
        let modifier = parse_field_modifier(walker)?;
        field_modifiers.push(modifier);
    }
    Ok((field_name, field_type, field_modifiers))
}
pub type FieldModifiers = Vec<FieldModifier>;
#[derive(Debug, Clone, PartialEq)]
pub enum FieldModifier {
    PrimaryKey,
    NotNull,
    Default(DataValue),
    AutoIncrement,
    Unique,
    Check(Expr),
}
/// Parses field modifier: PRIMARY KEY, UNIQUE, etc.
pub(super) fn parse_field_modifier(walker: &mut TokenWalker) -> ParseResult<FieldModifier> {
    let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if let TokenValue::Keyword(ident) = token {
        match ident.deref() {
            "UNIQUE" => Ok(FieldModifier::Unique),
            "AUTOINCREMENT" => Ok(FieldModifier::AutoIncrement),
            "PRIMARY" => {
                walker.expect_next_token(&TokenValue::Keyword("KEY".into()))?;
                Ok(FieldModifier::PrimaryKey)
            }
            "NOT" => {
                walker.expect_next_token(&TokenValue::Keyword("NULL".into()))?;
                Ok(FieldModifier::NotNull)
            }
            "DEFAULT" => {
                let default_value = parse_literal(walker)?;
                Ok(FieldModifier::Default(default_value))
            }
            _ => Err(ParseError::Other {
                message: format!("Unknown modifier {}", ident).into(),
            }),
        }
    } else {
        Err(ParseError::Other {
            message: format!(
                "No field modifiers provided. Also that's a bug. Current token: {:?}",
                token
            )
            .into(),
        })
    }
}
/// Parses INSERT INTO query
pub(super) fn parse_insert_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("INSERT".into()))?;
    walker.expect_next_token(&TokenValue::Keyword("INTO".into()))?;

    let table_name = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if !table_name.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "Name of the table".into(),
            given: table_name.to_string().into(),
        });
    }
    let table_name = table_name.to_string();
    let fields = parse_insert_fields(&mut walker)?;

    walker.expect_next_token(&TokenValue::Keyword("VALUES".into()))?;

    let mut insert_data = Vec::new();
    loop {
        let data = parse_insert_data(&mut walker)?;
        if data.len() != fields.len() {
            return Err(ParseError::Other {
                message: format!(
                    "number of fields not matches with number of values in row. Expected:{}, Provided:{}",
                    fields.len(),
                    data.len()
                ).into(),
            });
        }
        let mut map = HashMap::new();
        for i in 0..fields.len() {
            map.insert(fields[i].clone(), data[i].clone());
        }
        let type_value = SchemaValue::new(map);
        insert_data.push(type_value);
        if walker.next_meaningful() != Some(&TokenValue::Delimiter(Delimiter::Comma)) {
            break;
        }
    }
    Ok(Query::Insert(InsertQuery::new(table_name, insert_data)))
}
/// Parses field names in INSERT statement
pub(super) fn parse_insert_fields(walker: &mut TokenWalker) -> ParseResult<Vec<String>> {
    {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if !matches!(token, TokenValue::Delimiter(Delimiter::RoundOpen)) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "(".into(),
                given: token.to_string().into(),
            });
        }
    }
    let mut fields = Vec::new();
    loop {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if !token.is_ident() {
            return Err(ParseError::UnexpectedSymbol {
                expected: "Expected field name".into(),
                given: token.to_string().into(),
            });
        }
        fields.push(token.to_string());
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if token == &TokenValue::Delimiter(Delimiter::RoundClose) {
            break;
        }
        if token != &TokenValue::Delimiter(Delimiter::Comma) {
            return Err(ParseError::UnexpectedSymbol {
                expected: ",".into(),
                given: token.to_string().into(),
            });
        }
    }
    Ok(fields)
}
/// Parses arbitrary structure of this structure (Value,Value,Value)
///
/// Expects walker's pointer to be next to structure
pub(super) fn parse_insert_data(walker: &mut TokenWalker) -> ParseResult<Vec<DataValue>> {
    {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if !matches!(token, TokenValue::Delimiter(Delimiter::RoundOpen)) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "(".into(),
                given: token.to_string().into(),
            });
        }
    }
    let mut insert_data = Vec::new();
    loop {
        let data = parse_literal(walker)?;
        insert_data.push(data);
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if token == &TokenValue::Delimiter(Delimiter::RoundClose) {
            break;
        }
        if token != &TokenValue::Delimiter(Delimiter::Comma) {
            return Err(ParseError::UnexpectedSymbol {
                expected: ",".into(),
                given: token.to_string().into(),
            });
        }
    }
    Ok(insert_data)
}

pub(super) fn parse_truncate_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("TRUNCATE".into()))?;
    walker.expect_next_token(&TokenValue::Keyword("TABLE".into()))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit".into(),
            given: table_name_token.to_string().into(),
        });
    }
    let del_query = DeleteQuery::TruncateTable(TruncateTable::new(table_name_token.to_string()));
    Ok(Query::Delete(del_query))
}

pub(super) fn parse_drop_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("DROP".into()))?;
    walker.expect_next_token(&TokenValue::Keyword("TABLE".into()))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit".into(),
            given: table_name_token.to_string().into(),
        });
    }
    let del_query = DeleteQuery::DropTable(DropTable::new(table_name_token.to_string()));
    Ok(Query::Delete(del_query))
}

pub(super) fn parse_delete_query(mut walker: TokenWalker) -> ParseResult<Query> {
    walker.expect_next_token(&TokenValue::Keyword("DELETE".into()))?;
    walker.expect_next_token(&TokenValue::Keyword("FROM".into()))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit".into(),
            given: table_name_token.to_string().into(),
        });
    }
    let table_name = table_name_token.to_string();
    walker.expect_next_token(&TokenValue::Keyword("WHERE".into()))?;
    let end = walker.tokens().len();
    let expr = parse_expr(&mut walker, end)?;
    let del_query = DeleteQuery::DeleteRows(DeleteRows::new(table_name, expr));
    Ok(Query::Delete(del_query))
}
