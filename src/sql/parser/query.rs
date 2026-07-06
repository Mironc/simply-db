use std::collections::HashMap;

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
            common::{ParseError, ParseResult, TokenWalker, parse_field_name, parse_literal},
            expr::parse_expr,
            tokenizer::{Delimiter, TokenValue},
        },
        query::Query,
    },
};

use super::tokenizer::Sign;

pub fn parse_query<'a>(tokens: Vec<TokenValue<'a>>) -> ParseResult<'a, Query> {
    let walker = TokenWalker::new(&tokens);
    match walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnknownInstruction)?
    {
        TokenValue::Keyword("INSERT") => parse_insert_query(walker),
        TokenValue::Keyword("CREATE") => parse_create_query(walker),
        TokenValue::Keyword("SELECT") => parse_select_query(walker),
        TokenValue::Keyword("UPDATE") => parse_update_query(walker),
        TokenValue::Keyword("DROP") => parse_drop_query(walker),
        TokenValue::Keyword("TRUNCATE") => parse_truncate_query(walker),
        TokenValue::Keyword("DELETE") => parse_delete_query(walker),
        _ => Err(ParseError::UnknownInstruction),
    }
}
pub(super) fn parse_update_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("UPDATE"))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let table_name = table_name_token.to_string();

    walker.expect_next_token(&TokenValue::Keyword("SET"))?;
    let mut set_exprs = Vec::new();
    'outer: while let Some(token) = walker.peek_next_meaningful()
        && token != &TokenValue::Keyword("WHERE")
    {
        let field_name = parse_field_name(&mut walker)?;
        walker.expect_next_token(&TokenValue::Sign(Sign::Set))?;
        let mut clone = walker.clone();
        let mut next_token = walker.next_meaningful();
        while let Some(token) = next_token {
            if token == &TokenValue::Delimiter(Delimiter::Comma) {
                break;
            }
            if token == &TokenValue::Keyword("WHERE") {
                set_exprs.push((field_name, parse_expr(&mut clone, walker.position())?));
                break 'outer;
            }
            next_token = walker.next_meaningful();
        }
        set_exprs.push((field_name, parse_expr(&mut clone, walker.position())?));
    }
    let filter_expr = if let Some(token) = walker.current_token()
        && token == &TokenValue::Keyword("WHERE")
    {
        if token != &TokenValue::Keyword("WHERE") {
            return Err(ParseError::UnexpectedSymbol {
                expected: "WHERE clause",
                given: token.as_str(),
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
pub(super) fn parse_select_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("SELECT"))?;

    let projection = if walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnexpectedEof)?
        == &TokenValue::Sign(Sign::Asterisk)
    {
        walker.skip_meaningful(1);
        walker.expect_next_token(&TokenValue::Keyword("FROM"))?;
        Projection::Row
    } else {
        let mut expressions = Vec::new();
        let mut open = 0;
        let mut walker_new = walker.clone_simple();
        let mut token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        while token != &TokenValue::Keyword("FROM") {
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
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let table_name = table_name_token.to_string();
    let filter_expr = if let Some(TokenValue::Keyword(k)) = walker.peek_next_meaningful()
        && *k == "WHERE"
    {
        walker.skip(2);
        let end = walker.tokens().len();
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
pub(super) fn parse_create_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("CREATE"))?;
    walker.expect_next_token(&TokenValue::Keyword("TABLE"))?;

    let mut if_not_exists = false;
    if let (
        Some(TokenValue::Keyword("IF")),
        Some(TokenValue::Keyword("NOT")),
        Some(TokenValue::Keyword("EXISTS")),
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
        if token.is_ident() {
            if !token.starts_with_digit() {
                token.to_string()
            } else {
                return Err(ParseError::Other {
                    message: "Expected valid table name which not starts with digit",
                });
            }
        } else {
            return Err(ParseError::Other {
                message: "Expected table name",
            });
        }
    };
    let row_type = parse_create_fields(&mut walker)?;
    let create_table_query = CreateTable::new(table_name, row_type, if_not_exists);
    Ok(Query::CreateTable(create_table_query))
}
pub(super) fn parse_create_fields<'a>(walker: &mut TokenWalker<'a, '_>) -> ParseResult<'a, Schema> {
    let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if token != &TokenValue::Delimiter(Delimiter::RoundOpen) {
        return Err(ParseError::UnexpectedSymbol {
            expected: "(",
            given: token.as_str(),
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
                expected: "COMMA or )",
                given: token.as_str(),
            });
        }
    }

    Ok(Schema::new(fields))
}
pub(super) fn parse_field_and_modifiers<'a>(
    walker: &mut TokenWalker<'a, '_>,
) -> ParseResult<'a, (String, DataType, FieldModifiers)> {
    let field_name = parse_field_name(walker)?;
    let field_type = {
        let field_type_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if let Some(field_type) = ScalarType::from_str(&field_type_token.to_string()) {
            DataType::Scalar(field_type)
        } else {
            return Err(ParseError::Other {
                message: "Expected field type",
            });
        }
    };
    let mut field_modifiers = Vec::new();
    while let token = walker
        .peek_next_meaningful()
        .ok_or(ParseError::UnexpectedEof)?
        && (token.is_keyword() || token.is_ident())
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
pub(super) fn parse_field_modifier<'a>(
    walker: &mut TokenWalker<'a, '_>,
) -> ParseResult<'a, FieldModifier> {
    let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if let TokenValue::Keyword(ident) = token {
        match *ident {
            "UNIQUE" => Ok(FieldModifier::Unique),
            "AUTOINCREMENT" => Ok(FieldModifier::AutoIncrement),
            "PRIMARY" => {
                walker.expect_next_token(&TokenValue::Keyword("KEY"))?;
                Ok(FieldModifier::PrimaryKey)
            }
            "NOT" => {
                walker.expect_next_token(&TokenValue::Keyword("NULL"))?;
                Ok(FieldModifier::NotNull)
            }
            "DEFAULT" => {
                let default_value = parse_literal(walker)?;
                Ok(FieldModifier::Default(default_value))
            }
            _ => Err(ParseError::UnknownModifier { modifier: ident }),
        }
    } else {
        Err(ParseError::UnknownModifier {
            modifier: token.as_str(),
        })
    }
}
/// Parses INSERT INTO query
pub(super) fn parse_insert_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("INSERT"))?;
    walker.expect_next_token(&TokenValue::Keyword("INTO"))?;

    let table_name = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if !table_name.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "Name of the table",
            given: table_name.as_str(),
        });
    }
    let table_name = table_name.to_string();
    let fields = parse_insert_fields(&mut walker)?;

    walker.expect_next_token(&TokenValue::Keyword("VALUES"))?;

    let mut insert_data = Vec::new();
    loop {
        let data = parse_insert_data(&mut walker)?;
        if data.len() != fields.len() {
            return Err(ParseError::FieldNumberMismatch {
                expected: fields.len(),
                provided: data.len(),
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
pub(super) fn parse_insert_fields<'a>(
    walker: &mut TokenWalker<'a, '_>,
) -> ParseResult<'a, Vec<String>> {
    {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if !matches!(token, TokenValue::Delimiter(Delimiter::RoundOpen)) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "(",
                given: token.as_str(),
            });
        }
    }
    let mut fields = Vec::new();
    loop {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if !token.is_ident() {
            return Err(ParseError::UnexpectedSymbol {
                expected: "Expected field name",
                given: token.as_str(),
            });
        }
        fields.push(token.to_string());
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if token == &TokenValue::Delimiter(Delimiter::RoundClose) {
            break;
        }
        if token != &TokenValue::Delimiter(Delimiter::Comma) {
            return Err(ParseError::UnexpectedSymbol {
                expected: ",",
                given: token.as_str(),
            });
        }
    }
    Ok(fields)
}
/// Parses arbitrary structure of this structure (Value,Value,Value)
///
/// Expects walker's pointer to be next to structure
pub(super) fn parse_insert_data<'a>(
    walker: &mut TokenWalker<'a, '_>,
) -> ParseResult<'a, Vec<DataValue>> {
    {
        let token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
        if !matches!(token, TokenValue::Delimiter(Delimiter::RoundOpen)) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "(",
                given: token.as_str(),
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
                expected: ",",
                given: token.as_str(),
            });
        }
    }
    Ok(insert_data)
}

pub(super) fn parse_truncate_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("TRUNCATE"))?;
    walker.expect_next_token(&TokenValue::Keyword("TABLE"))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let del_query = DeleteQuery::TruncateTable(TruncateTable::new(table_name_token.to_string()));
    Ok(Query::Delete(del_query))
}

pub(super) fn parse_drop_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("DROP"))?;
    walker.expect_next_token(&TokenValue::Keyword("TABLE"))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let del_query = DeleteQuery::DropTable(DropTable::new(table_name_token.to_string()));
    Ok(Query::Delete(del_query))
}

pub(super) fn parse_delete_query<'a>(mut walker: TokenWalker<'a, '_>) -> ParseResult<'a, Query> {
    walker.expect_next_token(&TokenValue::Keyword("DELETE"))?;
    walker.expect_next_token(&TokenValue::Keyword("FROM"))?;
    let table_name_token = walker.next_meaningful().ok_or(ParseError::UnexpectedEof)?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let table_name = table_name_token.to_string();
    walker.expect_next_token(&TokenValue::Keyword("WHERE"))?;
    let end = walker.tokens().len();
    let expr = parse_expr(&mut walker, end)?;
    let del_query = DeleteQuery::DeleteRows(DeleteRows::new(table_name, expr));
    Ok(Query::Delete(del_query))
}
