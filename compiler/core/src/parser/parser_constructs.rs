use lalrpop_util::{lexer::Token, ParseError};

use super::{
    parse_location::Parsed,
    schema_ast::Entity,
    slang_ast::{Entrypoint, Fragment},
};
use crate::slang::TermParser;

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
