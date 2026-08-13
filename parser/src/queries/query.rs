use query::{
    Query,
    queries::{
        create_table::CreateTable,
        delete::{DeleteQuery, DeleteRows, DropTable, TruncateTable},
        insert::InsertQuery,
        select::{Projection, SelectQuery},
        update::UpdateQuery,
    },
};
use storage::{
    common_types::{DataValue, ScalarType},
    row::Row,
    schema::{FieldModifier, FieldType},
};
use structures::VecMap;

use crate::{
    common::{ParseError, ParseResult, Parser, parse_field_name, parse_literal},
    lexer::{Delimiter, Keyword, Lexer, TokenValue},
    queries::expr::parse_expr,
};

use crate::lexer::Sign;

pub fn parse_query<'a>(lexer: Lexer<'a>) -> ParseResult<'a, Query> {
    let parser = Parser::new(lexer)?;
    match parser.peek_next() {
        TokenValue::Keyword(Keyword::Insert) => parse_insert_query(parser),
        TokenValue::Keyword(Keyword::Create) => parse_create_query(parser),
        TokenValue::Keyword(Keyword::Select) => parse_select_query(parser),
        TokenValue::Keyword(Keyword::Update) => parse_update_query(parser),
        TokenValue::Keyword(Keyword::Drop) => parse_drop_query(parser),
        TokenValue::Keyword(Keyword::Truncate) => parse_truncate_query(parser),
        TokenValue::Keyword(Keyword::Delete) => parse_delete_query(parser),
        _ => Err(ParseError::UnknownInstruction),
    }
}
pub(super) fn parse_update_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Update))?;
    let table_name_token = parser.consume()?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let table_name = table_name_token.to_string();

    parser.expect_next_token(TokenValue::Keyword(Keyword::Set))?;

    let mut set_exprs = Vec::new();
    'outer: while parser.peek_next() != TokenValue::Keyword(Keyword::Where)
        && parser.peek_next() != TokenValue::EOF
    {
        let field_name = parse_field_name(&mut parser)?;
        parser.expect_next_token(TokenValue::Sign(Sign::Set))?;
        let mut clone = parser.clone();
        let mut next_token = parser.peek_next();
        while next_token != TokenValue::EOF {
            if next_token == TokenValue::Delimiter(Delimiter::Comma) {
                break;
            }
            if next_token == TokenValue::Keyword(Keyword::Where) {
                set_exprs.push((field_name, parse_expr(&mut clone, parser.position())?));
                clone.advance()?;
                break 'outer;
            }
            parser.advance()?;
            next_token = parser.peek_next();
        }
        set_exprs.push((field_name, parse_expr(&mut clone, parser.position())?));
        parser.advance()?;
    }
    let filter_expr = if parser.current_token() != TokenValue::EOF {
        parser.expect_next_token(TokenValue::Keyword(Keyword::Where))?;
        let end = parser.lexer().source().len();
        Some(parse_expr(&mut parser, end)?)
    } else {
        None
    };
    let query = UpdateQuery::new(table_name, set_exprs, filter_expr);
    Ok(Query::Update(query))
}
pub(super) fn parse_select_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Select))?;

    let projection = if parser.peek_next() == TokenValue::Sign(Sign::Asterisk) {
        parser.advance()?;
        Projection::Row
    } else {
        let mut expressions = Vec::new();
        let mut open = 0;
        let mut walker_new = parser.clone();
        let mut token = parser.peek_next();
        while token != TokenValue::Keyword(Keyword::From) && token != TokenValue::EOF {
            if token == TokenValue::Delimiter(Delimiter::RoundOpen) {
                open += 1
            }
            if token == TokenValue::Delimiter(Delimiter::RoundClose) {
                if open == 0 {
                    return Err(ParseError::UnclosedBracket(')'));
                }
                open -= 1
            }
            // Counting brackets to differentiate commas inside expressions and outside
            if token == TokenValue::Delimiter(Delimiter::Comma) && open == 0 {
                expressions.push(parse_expr(&mut walker_new, parser.position())?);
                walker_new.advance()?;
            }
            parser.advance()?;
            token = parser.peek_next();
        }
        expressions.push(parse_expr(&mut walker_new, parser.position())?);
        Projection::Expr(expressions)
    };
    parser.expect_next_token(TokenValue::Keyword(Keyword::From))?;
    let table_name_token = parser.consume()?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let table_name = table_name_token.to_string();
    let filter_expr = if parser.peek_next() == TokenValue::Keyword(Keyword::Where) {
        parser.advance()?; // Skip WHERE
        let mut copied = parser.clone();
        while !matches!(
            copied.consume()?,
            TokenValue::Keyword(Keyword::Take)
                | TokenValue::Keyword(Keyword::Skip)
                | TokenValue::Keyword(Keyword::Order)
                | TokenValue::EOF
        ) {}
        let expr_end = copied.position();
        Some(parse_expr(&mut parser, expr_end)?)
    } else {
        None
    };
    let mut skip = None;
    let mut take = None;
    while let TokenValue::Keyword(keyword) = parser.consume()? {
        match keyword {
            Keyword::Take => {
                take = Some(parse_parameter_number_argument(&mut parser)?);
            }
            Keyword::Skip => {
                skip = Some(parse_parameter_number_argument(&mut parser)?);
            }
            Keyword::Order => {}
            _ => (),
        }
    }
    Ok(Query::Select(SelectQuery::new(
        table_name,
        projection,
        filter_expr,
        skip,
        take,
    )))
}
fn parse_parameter_number_argument<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, usize> {
    let token = parser.consume()?;
    if let TokenValue::Sign(Sign::Minus) = token {
        return Err(ParseError::Other {
            message: "Expected non-negative argument",
        });
    }
    let result = if let TokenValue::Ident(word) = token {
        match str::parse::<usize>(word) {
            Ok(n) => n,
            Err(_) => {
                return Err(ParseError::Other {
                    message: "Expected number literal",
                });
            }
        }
    } else {
        return Err(ParseError::UnexpectedValue {
            expected: "Expected numeric argument",
        });
    };
    if let TokenValue::Delimiter(Delimiter::Dot) = parser.peek_next() {
        return Err(ParseError::UnexpectedValue {
            expected: "Expected integer argument",
        });
    }

    Ok(result)
}
/// Parses CREATE TABLE query
pub(super) fn parse_create_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Create))?;
    parser.expect_next_token(TokenValue::Keyword(Keyword::Table))?;

    let mut if_not_exists = false;
    if parser.peek_next() == TokenValue::Keyword(Keyword::If) {
        parser.advance()?;

        if parser.peek_next() == TokenValue::Keyword(Keyword::Not) {
            parser.advance()?;

            if parser.peek_next() == TokenValue::Keyword(Keyword::Exists) {
                parser.advance()?;
                if_not_exists = true;
            }
        }
    }

    let table_name = {
        let token = parser.consume()?;
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
    let row_type = parse_create_fields(&mut parser)?;
    let create_table_query = CreateTable::new(table_name, row_type, if_not_exists);
    Ok(Query::CreateTable(create_table_query))
}
pub(super) fn parse_create_fields<'a>(
    parser: &mut Parser<'a>,
) -> ParseResult<'a, VecMap<String, FieldType>> {
    let token = parser.consume()?;
    if token != TokenValue::Delimiter(Delimiter::RoundOpen) {
        return Err(ParseError::UnexpectedSymbol {
            expected: "(",
            given: token.as_str(),
        });
    }
    let mut fields = VecMap::new();
    loop {
        let fields_parsed = parse_field_and_modifiers(parser)?;
        // TODO: FIELD MODIFIERS
        let field = FieldType::new(fields_parsed.1, fields_parsed.2);
        fields.insert(fields_parsed.0, field);
        let token = parser.consume()?;
        if let TokenValue::Delimiter(Delimiter::RoundClose) = token {
            break;
        }
        if token != TokenValue::Delimiter(Delimiter::Comma) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "COMMA or )",
                given: token.as_str(),
            });
        }
    }

    Ok(fields)
}
pub(super) fn parse_field_and_modifiers<'a>(
    walker: &mut Parser<'a>,
) -> ParseResult<'a, (String, ScalarType, FieldModifiers)> {
    let field_name = parse_field_name(walker)?;
    let field_type = {
        let field_type_token = walker.consume()?;
        if let Some(field_type) = ScalarType::from_str(&field_type_token.to_string()) {
            field_type
        } else {
            return Err(ParseError::Other {
                message: "Expected field type",
            });
        }
    };
    let mut field_modifiers = Vec::new();
    while let token = walker.peek_next()
        && (token.is_keyword() || token.is_ident())
    {
        let modifier = parse_field_modifier(walker)?;
        field_modifiers.push(modifier);
    }
    Ok((field_name, field_type, field_modifiers))
}
pub type FieldModifiers = Vec<FieldModifier>;
/// Parses field modifier: PRIMARY KEY, UNIQUE, etc.
pub(super) fn parse_field_modifier<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, FieldModifier> {
    let token = parser.consume()?;
    if let TokenValue::Keyword(ident) = token {
        match ident {
            Keyword::Unique => Ok(FieldModifier::Unique),
            Keyword::AutoIncrement => Ok(FieldModifier::AutoIncrement),
            Keyword::Primary => {
                parser.expect_next_token(TokenValue::Keyword(Keyword::Key))?;
                Ok(FieldModifier::PrimaryKey)
            }
            Keyword::Not => {
                parser.expect_next_token(TokenValue::Keyword(Keyword::Null))?;
                Ok(FieldModifier::NotNull)
            }
            Keyword::Default => {
                let default_value = parse_literal(parser)?;
                Ok(FieldModifier::Default(default_value))
            }
            _ => Err(ParseError::UnknownModifier {
                modifier: ident.as_str(),
            }),
        }
    } else {
        Err(ParseError::UnknownModifier {
            modifier: token.as_str(),
        })
    }
}
/// Parses INSERT INTO query
pub(super) fn parse_insert_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Insert))?;
    parser.expect_next_token(TokenValue::Keyword(Keyword::Into))?;

    let table_name = parser.consume()?;
    if !table_name.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "Name of the table",
            given: table_name.as_str(),
        });
    }
    let table_name = table_name.to_string();
    let fields = parse_insert_fields(&mut parser)?;

    parser.expect_next_token(TokenValue::Keyword(Keyword::Values))?;

    let mut insert_data = Vec::new();
    loop {
        let data = parse_insert_data(&mut parser)?;
        if data.len() != fields.len() {
            return Err(ParseError::FieldNumberMismatch {
                expected: fields.len(),
                provided: data.len(),
            });
        }
        let type_value = Row::new(data);
        insert_data.push(type_value);
        if let token = parser.consume()?
            && (token != TokenValue::Delimiter(Delimiter::Comma))
        {
            break;
        }
    }
    Ok(Query::Insert(InsertQuery::new(
        table_name,
        fields,
        insert_data,
    )))
}
/// Parses field names in INSERT statement
///
#[inline]
pub(super) fn parse_insert_fields<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Vec<String>> {
    {
        let token = parser.consume()?;
        if !matches!(token, TokenValue::Delimiter(Delimiter::RoundOpen)) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "(",
                given: token.as_str(),
            });
        }
    }
    let mut fields = Vec::new();
    loop {
        let token = parser.consume()?;
        if !token.is_ident() {
            return Err(ParseError::UnexpectedSymbol {
                expected: "Expected field name",
                given: token.as_str(),
            });
        }
        fields.push(token.to_string());
        let token = parser.consume()?;
        if token == TokenValue::Delimiter(Delimiter::RoundClose) {
            break;
        }
        if token != TokenValue::Delimiter(Delimiter::Comma) {
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
/// Expects walker's pointer to be next_meaningful to structure
#[inline]
pub(super) fn parse_insert_data<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Vec<DataValue>> {
    {
        let token = parser.consume()?;
        if !matches!(token, TokenValue::Delimiter(Delimiter::RoundOpen)) {
            return Err(ParseError::UnexpectedSymbol {
                expected: "(",
                given: token.as_str(),
            });
        }
    }
    let mut insert_data = Vec::new();
    loop {
        let data = parse_literal(parser)?;
        insert_data.push(data);
        let token = parser.consume()?;
        if token == TokenValue::Delimiter(Delimiter::RoundClose) {
            break;
        }
        if token != TokenValue::Delimiter(Delimiter::Comma) {
            return Err(ParseError::UnexpectedSymbol {
                expected: ",",
                given: token.as_str(),
            });
        }
    }
    Ok(insert_data)
}

pub(super) fn parse_truncate_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Truncate))?;
    parser.expect_next_token(TokenValue::Keyword(Keyword::Table))?;
    let table_name_token = parser.consume()?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let del_query = DeleteQuery::TruncateTable(TruncateTable::new(table_name_token.to_string()));
    Ok(Query::Delete(del_query))
}

pub(super) fn parse_drop_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Drop))?;
    parser.expect_next_token(TokenValue::Keyword(Keyword::Table))?;
    let table_name_token = parser.consume()?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let del_query = DeleteQuery::DropTable(DropTable::new(table_name_token.to_string()));
    Ok(Query::Delete(del_query))
}

pub(super) fn parse_delete_query<'a>(mut parser: Parser<'a>) -> ParseResult<'a, Query> {
    parser.expect_next_token(TokenValue::Keyword(Keyword::Delete))?;
    parser.expect_next_token(TokenValue::Keyword(Keyword::From))?;
    let table_name_token = parser.consume()?;
    if table_name_token.starts_with_digit() || !table_name_token.is_ident() {
        return Err(ParseError::UnexpectedSymbol {
            expected: "valid table name that starts not with digit",
            given: table_name_token.as_str(),
        });
    }
    let table_name = table_name_token.to_string();
    parser.expect_next_token(TokenValue::Keyword(Keyword::Where))?;
    let end = parser.lexer().source().len();
    let expr = parse_expr(&mut parser, end)?;
    let del_query = DeleteQuery::DeleteRows(DeleteRows::new(table_name, expr));
    Ok(Query::Delete(del_query))
}
