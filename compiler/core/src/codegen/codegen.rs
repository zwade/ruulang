use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    config::config::RuuLangConfig,
    parser::{
        parse_location::Parsed,
        ruulang_ast::{Attribute, Entrypoint, Fragment, RuuLangFile},
        schema_ast::{Entity, Relationship},
    },
    utils::with_origin::WithOrigin,
};

pub struct CodegenState<Import>
where
    Import: Clone + Eq + core::hash::Hash + std::fmt::Debug,
{
    imports: HashSet<Import>,
    exports: HashSet<String>,

    code_blocks: Vec<String>,
}

impl<Import> CodegenState<Import>
where
    Import: Clone + Eq + core::hash::Hash + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            imports: HashSet::new(),
            exports: HashSet::new(),
            code_blocks: vec![],
        }
    }

    pub fn add_import(&mut self, import: Import) {
        self.imports.insert(import);
    }

    pub fn write_code(&mut self, code: String) {
        self.code_blocks.push(code);
    }

    pub fn concat(self, other: CodegenState<Import>) -> Self {
        let mut imports = self.imports;
        imports.extend(other.imports);

        let mut exports = self.exports;
        exports.extend(other.exports);

        let mut code_blocks = self.code_blocks;
        code_blocks.extend(other.code_blocks);

        Self {
            imports,
            exports,
            code_blocks,
        }
    }

    pub fn stringify<'a>(self, codegen: &impl Codegen<'a, Import>) -> String {
        let mut output = vec![];

        if let Some(result) = codegen.serialize_prelude() {
            output.push(result);
        }

        if self.imports.len() > 0 {
            let (entities, _) = codegen.get_schema_and_file();
            let entity_map = entities.iter().fold(HashMap::new(), |mut acc, entity| {
                let new_file_name = entity.origin.with_extension("");
                acc.insert(entity.data.data.name.data.as_str(), new_file_name);

                acc
            });

            if let Some(result) =
                codegen.serialize_imports(&self.imports.iter().collect(), &entity_map)
            {
                output.push(result);
            }
        }

        for code_block in self.code_blocks {
            output.push(code_block);
        }

        for export in self.exports {
            codegen.serialize_export(&export).and_then(|export| {
                output.push(export);
                Some(())
            });
        }

        output.join("\n")
    }
}

pub trait Codegen<'a, Import>
where
    Import: Clone + Eq + core::hash::Hash + std::fmt::Debug,
{
    fn new(
        origin: &'a PathBuf,
        file_name: &'a PathBuf,
        config: &'a RuuLangConfig,
        entities: &'a Vec<WithOrigin<Parsed<Entity>>>,
        file: &'a RuuLangFile,
    ) -> Self;
    fn get_schema_and_file(&self) -> (&'a Vec<WithOrigin<Parsed<Entity>>>, &'a RuuLangFile);
    fn get_origin(&self) -> &'a PathBuf;

    fn serialize_imports(
        &self,
        _imports: &Vec<&Import>,
        _entity_map: &HashMap<&str, PathBuf>,
    ) -> Option<String> {
        None
    }

    fn serialize_prelude(&self) -> Option<String> {
        None
    }

    fn serialize_header(&self) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_grant(
        &self,
        _entity: &Entity,
        _grant: &Vec<String>,
    ) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_attribute(
        &self,
        _entity: &Entity,
        _rule: &Relationship,
        _attribute: &Attribute,
    ) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_relationship(
        &self,
        _entity: &Entity,
        _rule: &Relationship,
    ) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_fragment(&self, _fragment: &Fragment) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_entrypoint(&self, _entrypoint: &Entrypoint) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_entity(&self, _entity: &Entity) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_footer(&self) -> Option<CodegenState<Import>> {
        None
    }

    fn serialize_export(&self, _export: &String) -> Option<String> {
        None
    }

    fn serialize_schema_and_file(&self) -> String
    where
        Self: Sized,
    {
        let mut state: CodegenState<Import> = CodegenState::new();
        let (entities, file) = self.get_schema_and_file();
        let origin = self.get_origin();

        if let Some(new_state) = self.serialize_header() {
            state = state.concat(new_state);
        }

        for entity in entities {
            if &entity.origin != origin {
                continue;
            }

            for grant in &entity.data.data.grants {
                if let Some(new_state) = self.serialize_grant(&entity.data.data, &grant.data) {
                    state = state.concat(new_state);
                }
            }

            for rule in &entity.data.data.relationships {
                for attr in &rule.data.attributes {
                    if let Some(new_state) =
                        self.serialize_attribute(&entity.data.data, &rule.data, &attr.data)
                    {
                        state = state.concat(new_state);
                    }
                }

                if let Some(new_state) = self.serialize_relationship(&entity.data.data, &rule.data)
                {
                    state = state.concat(new_state);
                }
            }

            if let Some(new_state) = self.serialize_entity(&entity.data.data) {
                state = state.concat(new_state);
            }
        }

        for entrypoint in &file.entrypoints {
            if let Some(new_state) = self.serialize_entrypoint(&entrypoint.data) {
                state = state.concat(new_state);
            }
        }

        for fragment in &file.fragments {
            if let Some(new_state) = self.serialize_fragment(&fragment.data) {
                state = state.concat(new_state);
            }
        }

        if let Some(new_state) = self.serialize_footer() {
            state = state.concat(new_state);
        }

        state.stringify(self)
    }
}
