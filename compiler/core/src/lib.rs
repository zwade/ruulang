#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub slang, "/parser/slang.rs"); // synthesized by LALRPOP

pub mod parser {
    pub mod slang_ast;
}
