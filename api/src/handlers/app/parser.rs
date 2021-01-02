use combine::{ParseError, Parser, Stream, attempt, between, choice, eof, from_str, look_ahead, many, many1, parser, satisfy, token};
use combine::parser::char::{digit, string};
use combine::error::StreamError;
use std::str::FromStr;

use crate::errors;
use super::text;

impl FromStr for text::Text {
    type Err = errors::ServiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Ok(a_text().parse(s)?.0)
        Ok(text::Text::ReqTasks(text::ReqTasks { tasks: Vec::new() }))
    }
}
