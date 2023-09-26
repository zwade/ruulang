use std::io;

use lalrpop_util::{lexer::Token, ParseError};

use crate::parser::{parse_location::Parsed, schema_ast::Relationship};

#[derive(Debug, Clone)]
pub enum TypecheckError {
    DuplicateRelationship(Parsed<Relationship>),
    GeneralError(Parsed<String>),
}

#[derive(Debug, Clone)]
pub enum SlangError {
    FileNotFound(String),
    SerdeParseError(toml::de::Error),
    SlangParseError(usize),
    TypecheckError(TypecheckError),
    Other(&'static str),
}

pub type Result<T> = std::result::Result<T, SlangError>;

impl From<io::Error> for SlangError {
    fn from(value: io::Error) -> Self {
        SlangError::FileNotFound(value.to_string())
    }
}

impl From<toml::de::Error> for SlangError {
    fn from(value: toml::de::Error) -> Self {
        SlangError::SerdeParseError(value)
    }
}

impl<'a> From<ParseError<usize, Token<'a>, &'static str>> for SlangError {
    fn from(value: ParseError<usize, Token<'a>, &'static str>) -> Self {
        match value {
            ParseError::ExtraToken { token: (start, ..) } => SlangError::SlangParseError(start),
            ParseError::InvalidToken { location } => SlangError::SlangParseError(location),
            ParseError::UnrecognizedEOF { location, .. } => SlangError::SlangParseError(location),
            ParseError::UnrecognizedToken {
                token: (start, ..), ..
            } => SlangError::SlangParseError(start),
            ParseError::User { error } => SlangError::Other(error),
        }
    }
}
