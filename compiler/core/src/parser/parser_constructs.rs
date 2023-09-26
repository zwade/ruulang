use lalrpop_util::{lexer::Token, ParseError};

use super::{
    parse_location::Parsed,
    ruulang_ast::{Entrypoint, Fragment},
    schema_ast::Entity,
};
use crate::ruulang::TermParser;

pub enum ParserStatement {
    Comment(String),
    Fragment(Parsed<Fragment>),
    Entrypoint(Parsed<Entrypoint>),
    Entity(Parsed<Entity>),
}

impl ParserStatement {
    pub fn parse(input: &str) -> Result<Vec<Self>, ParseError<usize, Token<'_>, &'static str>> {
        TermParser::new().parse(input)
    }
}
