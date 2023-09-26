use std::io;

use lalrpop_util::{lexer::Token, ParseError};

use crate::parser::{parse_location::Parsed, schema_ast::Relationship};

#[derive(Debug, Clone)]
pub enum TypecheckError {
    DuplicateRelationship(Parsed<Relationship>),
    GeneralError(Parsed<String>),
}

#[derive(Debug, Clone)]
pub enum RuuLangError {
    FileNotFound(String),
    SerdeParseError(toml::de::Error),
    RuuLangParseError(usize),
    TypecheckError(TypecheckError),
    Other(&'static str),
}

pub type Result<T> = std::result::Result<T, RuuLangError>;

impl From<io::Error> for RuuLangError {
    fn from(value: io::Error) -> Self {
        RuuLangError::FileNotFound(value.to_string())
    }
}

impl From<toml::de::Error> for RuuLangError {
    fn from(value: toml::de::Error) -> Self {
        RuuLangError::SerdeParseError(value)
    }
}

impl<'a> From<ParseError<usize, Token<'a>, &'static str>> for RuuLangError {
    fn from(value: ParseError<usize, Token<'a>, &'static str>) -> Self {
        match value {
            ParseError::ExtraToken { token: (start, ..) } => RuuLangError::RuuLangParseError(start),
            ParseError::InvalidToken { location } => RuuLangError::RuuLangParseError(location),
            ParseError::UnrecognizedEOF { location, .. } => {
                RuuLangError::RuuLangParseError(location)
            }
            ParseError::UnrecognizedToken {
                token: (start, ..), ..
            } => RuuLangError::RuuLangParseError(start),
            ParseError::User { error } => RuuLangError::Other(error),
        }
    }
}
