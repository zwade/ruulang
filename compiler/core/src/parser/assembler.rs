use super::{parser_constructs::ParserStatement, schema_ast::SlangSchema, slang_ast::SlangFile};

pub trait ParserAssemble {
    fn assemble(&self) -> (SlangSchema, SlangFile);
}

impl ParserAssemble for Vec<ParserStatement> {
    fn assemble(&self) -> (SlangSchema, SlangFile) {
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

        let schema = SlangSchema { entities };
        let file = SlangFile {
            entrypoints,
            fragments,
        };

        (schema, file)
    }
}
