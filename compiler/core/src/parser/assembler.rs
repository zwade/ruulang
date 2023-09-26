use super::{
    parser_constructs::ParserStatement, ruulang_ast::RuuLangFile, schema_ast::RuuLangSchema,
};

pub trait ParserAssemble {
    fn assemble(&self) -> (RuuLangSchema, RuuLangFile);
}

impl ParserAssemble for Vec<ParserStatement> {
    fn assemble(&self) -> (RuuLangSchema, RuuLangFile) {
        let mut fragments = Vec::new();
        let mut entrypoints = Vec::new();
        let mut entities = Vec::new();

        for statement in self {
            match statement {
                ParserStatement::Comment(_) => {}
                ParserStatement::Fragment(fragment) => {
                    fragments.push(fragment.clone());
                }
                ParserStatement::Entrypoint(entrypoint) => {
                    entrypoints.push(entrypoint.clone());
                }
                ParserStatement::Entity(entity) => {
                    entities.push(entity.clone());
                }
            }
        }

        let schema = RuuLangSchema { entities };
        let file = RuuLangFile {
            entrypoints,
            fragments,
        };

        (schema, file)
    }
}
