#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub ruulang, "/parser/ruulang.rs"); // synthesized by LALRPOP

pub mod parser {
    pub mod assembler;
    pub mod parse_location;
    pub mod parser_constructs;
    pub mod parser_utils;
    pub mod ruulang_ast;
    pub mod schema_ast;
}

pub mod config {
    pub mod config;
}

pub mod workspace {
    pub mod workspace;
}

pub mod utils {
    pub mod error;
    pub mod trie;
    pub mod with_origin;
}

pub mod typechecker {
    mod tc_ast;
    pub mod typechecker;
}

pub mod codegen {
    pub mod codegen;
    pub mod python;

    mod codegen_helper;
    mod codegen_utils;
}
