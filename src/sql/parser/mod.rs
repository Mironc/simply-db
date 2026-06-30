//! This parser contains everything related to parsing SQL queries
pub mod common;
#[cfg(test)]
mod create_table_tests;
pub mod expr;
#[cfg(test)]
mod expr_tests;
#[cfg(test)]
mod insert_tests;
pub mod query;
#[cfg(test)]
mod select_tests;
pub mod tokenizer;
#[cfg(test)]
mod update_tests;
