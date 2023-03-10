use lalrpop_util::{ParseError, lexer::Token};

use super::slang_ast::{Entrypoint, SlangFile, Fragment};
use crate::slang::TermParser;

pub enum ParserStatement {
    Comment(String),
    Fragment(Fragment),
    Entrypoint(Entrypoint),
}

impl ParserStatement {
    pub fn parse(input: &str) -> Result<Vec<Self>, ParseError<usize, Token<'_>, &'static str>> {
        TermParser::new().parse(input)
    }
}


pub trait ParserAssemble {
    fn assemble(&self) -> SlangFile;
}

impl ParserAssemble for Vec<ParserStatement> {
    fn assemble(&self) -> SlangFile {
        let mut fragments = Vec::new();
        let mut entrypoints = Vec::new();

        for statement in self {
            match statement {
                ParserStatement::Comment(_) => {}
                ParserStatement::Fragment(fragment) => {
                    fragments.push(fragment.clone());
                }
                ParserStatement::Entrypoint(entrypoint) => {
                    entrypoints.push(entrypoint.clone());
                }
            }
        }

        SlangFile { entrypoints, fragments }
    }
}