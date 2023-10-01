use std::{collections::HashMap, path::PathBuf};

use crate::{
    config::config::RuuLangConfig,
    parser::{
        parse_location::Parsed,
        ruulang_ast::{Attribute, Entrypoint, Fragment, RuuLangFile},
        schema_ast::{Entity, Relationship},
    },
    utils::with_origin::WithOrigin,
};

use super::{
    codegen::{Codegen, CodegenState},
    codegen_helper::CodegenHelper,
    codegen_utils,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PythonImport {
    Global((String, Option<String>)),
    LocalImport((String, String)), // (entity, value)
}

impl PythonImport {
    pub fn new_global(module: &str, name: &str) -> Self {
        Self::Global((module.to_string(), Some(name.to_string())))
    }

    pub fn new_global_module(module: &str) -> Self {
        Self::Global((module.to_string(), None))
    }

    pub fn new_local(entity: &str, value: &str) -> Self {
        Self::LocalImport((entity.to_string(), value.to_string()))
    }

    fn with_class(
        s: &mut CodegenHelper<'_>,
        name: &str,
        subclasses: Vec<&str>,
        op: impl FnOnce(&mut CodegenHelper<'_>),
    ) {
        s.write_token("class");
        s.write_token(name);

        if subclasses.len() > 0 {
            s.with_parens(|s| s.iter_and_join(subclasses, ", ", |s, cls| s.write(cls)))
        }

        s.write_line(Some(":"));
        s.with_indent(op);
    }
}

pub struct PythonCodegen<'a> {
    origin: &'a PathBuf,
    file_name: &'a PathBuf,
    entities: &'a Vec<WithOrigin<Parsed<Entity>>>,
    config: &'a RuuLangConfig,
    file: &'a RuuLangFile,
}

impl<'a> PythonCodegen<'a> {
    fn new_codegen_helper(&self) -> CodegenHelper<'a> {
        CodegenHelper::new("    ", "\n")
    }
}

impl<'a> Codegen<'a, PythonImport> for PythonCodegen<'a> {
    fn new(
        origin: &'a PathBuf,
        file_name: &'a PathBuf,
        config: &'a RuuLangConfig,
        entities: &'a Vec<WithOrigin<Parsed<Entity>>>,
        file: &'a RuuLangFile,
    ) -> Self {
        Self {
            origin,
            file_name,
            config,
            entities,
            file,
        }
    }

    fn get_schema_and_file(&self) -> (&'a Vec<WithOrigin<Parsed<Entity>>>, &'a RuuLangFile) {
        (self.entities, self.file)
    }

    fn get_origin(&self) -> &'a PathBuf {
        self.origin
    }

    fn serialize_imports(
        &self,
        imports: &Vec<&PythonImport>,
        entity_map: &HashMap<&str, PathBuf>,
    ) -> Option<String> {
        let mut s = self.new_codegen_helper();
        let mut global_imports = vec![];
        let mut global_imports_by_module = HashMap::<String, Vec<&String>>::new();
        let mut local_imports_by_module = HashMap::<String, Vec<&String>>::new();

        for import in imports {
            match import {
                PythonImport::Global((module, None)) => {
                    global_imports.push(module);
                }

                PythonImport::Global((module, Some(name))) => {
                    let entry = global_imports_by_module.entry(module.clone());
                    entry.or_default().push(name);
                }

                PythonImport::LocalImport((entity, value)) => {
                    let entity_loc = entity_map.get(entity.as_str());
                    let local_extns = self.file_name.with_extension("");

                    match entity_loc {
                        None => {}
                        Some(path) if path == &local_extns => {}
                        Some(path) => {
                            let local_path = path
                                .strip_prefix(&self.config.workspace.root.as_ref().unwrap())
                                .unwrap();

                            let module = local_path
                                .components()
                                .map(|x| x.as_os_str().to_str().unwrap())
                                .collect::<Vec<_>>()
                                .join(".");

                            let entry = local_imports_by_module.entry(module);
                            entry.or_default().push(value);
                        }
                    }
                }
            }
        }

        if global_imports.len() > 0 {
            global_imports.sort();

            for import in global_imports {
                s.write(format!("import {}\n", import).as_str());
            }

            s.write("\n");
        }

        if global_imports_by_module.len() > 0 {
            let mut keys = global_imports_by_module.keys().collect::<Vec<_>>();
            keys.sort();

            for key in keys {
                s.write(format!("from {} import ", key).as_str());

                let mut values = global_imports_by_module.get(key).unwrap().clone();
                values.sort();

                for (i, value) in values.iter().enumerate() {
                    s.write(value);

                    if i < values.len() - 1 {
                        s.write(", ");
                    }
                }

                s.write("\n");
            }

            s.write("\n");
        }

        if local_imports_by_module.len() > 0 {
            let mut keys = local_imports_by_module.keys().collect::<Vec<_>>();
            keys.sort();

            for key in keys {
                s.write(format!("from {} import ", key).as_str());

                let mut values = local_imports_by_module.get(key).unwrap().clone();
                values.sort();

                for (i, value) in values.iter().enumerate() {
                    s.write(value);

                    if i < values.len() - 1 {
                        s.write(", ");
                    }
                }

                s.write("\n");
            }

            s.write("\n");
        }

        Some(s.serialize())
    }

    fn serialize_attribute(
        &self,
        entity: &Entity,
        rel: &Relationship,
        attribute: &Attribute,
    ) -> Option<CodegenState<PythonImport>> {
        let mut s = self.new_codegen_helper();
        let attr_name = format!(
            "{}{}{}Attr",
            &codegen_utils::camel_case(&entity.name),
            &codegen_utils::camel_case(&rel.relationship_name),
            &codegen_utils::camel_case(&attribute.name)
        );

        s.write_line(Some(
            format!(
                "@registry.register_attribute(\"{}\", \"{}\", \"{}\")",
                &entity.name, &rel.relationship_name, &attribute.name
            )
            .as_str(),
        ));
        PythonImport::with_class(&mut s, attr_name.as_str(), vec!["Attribute"], |s| {
            s.write("name: Literal[");
            s.with_duouble_quote(|s| s.write(&attribute.name));
            s.write("]");
            s.write_line(None);
        });

        let mut state = CodegenState::new();
        state.add_import(PythonImport::new_global("ruu_runtime", "Attribute"));
        state.add_import(PythonImport::new_global("ruu_runtime", "registry"));
        state.add_import(PythonImport::new_global("typing", "Literal"));
        state.write_code(s.serialize());
        Some(state)
    }

    fn serialize_relationship(
        &self,
        entity: &Entity,
        rule: &Relationship,
    ) -> Option<CodegenState<PythonImport>> {
        let mut state = CodegenState::new();
        let mut s = self.new_codegen_helper();
        let rel_name = format!(
            "{}{}Rule",
            &codegen_utils::camel_case(&entity.name),
            &codegen_utils::camel_case(&rule.relationship_name),
        );

        s.write_line(Some(
            format!(
                "@registry.register_relationship(\"{}\", \"{}\", \"{}\")",
                &entity.name, &rule.relationship_name, &rule.entity_name,
            )
            .as_str(),
        ));
        PythonImport::with_class(&mut s, &rel_name, vec!["Rule"], |s| {
            s.write("relationship: Literal[");
            s.with_duouble_quote(|s| s.write(&rule.relationship_name));
            s.write("]");
            s.write_line(None);

            let src_entity = self
                .entities
                .iter()
                .find(|e| e.data.data.name == rule.entity_name);

            let dst_entity = src_entity;

            let def = vec![];
            let grants = dst_entity.map_or(&def, |e| &e.data.data.grants);

            if grants.len() == 0 {
                s.write("grants: tuple[()]");
            } else {
                s.write("grants: tuple[");
                s.iter_and_join(grants, " | ", |s, grant| {
                    s.write_token("tuple");
                    s.write_symbol("[");
                    s.iter_and_join(&grant.data.grant, ", ", |s, g| {
                        s.write("Literal[");
                        s.with_duouble_quote(|s| {
                            s.write(&g);
                        });
                        s.write("]");
                    });
                    s.write_symbol("]");
                });

                s.write(", ...]");
            }
            s.write_line(None);

            if rule.attributes.len() == 0 {
                s.write("attributes: tuple[()]")
            } else {
                s.write("attributes: \"tuple[");
                s.iter_and_join(&rule.attributes, " | ", |s, attr| {
                    let attr_name = format!(
                        "{}{}{}Attr",
                        &codegen_utils::camel_case(&entity.name),
                        &codegen_utils::camel_case(&rule.relationship_name),
                        &codegen_utils::camel_case(&attr.data.name)
                    );

                    s.write(&attr_name);
                });
                s.write(", ...]\"")
            };
            s.write_line(None);

            let def = vec![];
            let rules = dst_entity.map_or(&def, |e| &e.data.data.relationships);

            if rules.len() == 0 {
                s.write("rules: tuple[Universal, ...]")
            } else {
                let entity = dst_entity.unwrap();

                s.write("rules: \"tuple[Universal | ");
                s.iter_and_join(rules, " | ", |s, rule| {
                    let rel_name = format!(
                        "{}{}Rule",
                        &codegen_utils::camel_case(&entity.data.data.name),
                        &codegen_utils::camel_case(&rule.data.relationship_name),
                    );

                    s.write(&rel_name);
                    state.add_import(PythonImport::new_local(&entity.data.data.name, &rel_name));
                });
                s.write(", ...]\"")
            };
            s.write_line(None);
        });

        state.add_import(PythonImport::new_global("ruu_runtime", "Rule"));
        state.add_import(PythonImport::new_global("ruu_runtime", "Universal"));
        state.add_import(PythonImport::new_global("ruu_runtime", "registry"));
        state.add_import(PythonImport::new_global("typing", "Literal"));
        state.write_code(s.serialize());
        Some(state)
    }

    fn serialize_fragment(&self, fragment: &Fragment) -> Option<CodegenState<PythonImport>> {
        let mut s = CodegenHelper::new("    ", "\n");

        let cls_name = format!(
            "{}{}Fragment",
            &codegen_utils::camel_case(&fragment.for_entity),
            &codegen_utils::camel_case(&fragment.name)
        );

        s.write_line(Some(
            format!(
                "@registry.register_fragment(\"{}\", \"{}\")",
                &fragment.for_entity, &fragment.name
            )
            .as_str(),
        ));
        PythonImport::with_class(&mut s, &cls_name, vec!["Fragment"], |s| {
            s.write_token("grants");
            s.write_symbol(": ");
            s.write_symbol("tuple[");

            s.iter_and_join(&fragment.grants, " | ", |s, grant| {
                s.write_token("tuple");
                s.write_symbol("[");
                s.iter_and_join(&grant.data.grant, ", ", |s, g| {
                    s.write("Literal[");
                    s.with_duouble_quote(|s| {
                        s.write(&g);
                    });
                    s.write("]");
                });
                s.write_symbol("]");
            });

            s.write_symbol(", ...]");
            s.write_line(None);
        });

        let mut state = CodegenState::new();
        state.write_code(s.serialize());
        state.add_import(PythonImport::new_global("ruu_runtime", "Fragment"));
        state.add_import(PythonImport::new_global("typing", "Literal"));

        Some(state)
    }

    fn serialize_entrypoint(&self, entrypoint: &Entrypoint) -> Option<CodegenState<PythonImport>> {
        let mut s = self.new_codegen_helper();
        let mut state = CodegenState::new();

        let name = format!(
            "{}Entrypoint",
            &codegen_utils::camel_case(&entrypoint.entrypoint)
        );

        s.write_line(Some("@registry.bind"));
        PythonImport::with_class(&mut s, &name, vec!["Entrypoint"], |s| {
            s.write_token("entrypoint");
            s.write_symbol(": ");
            s.write_symbol("Literal[");
            s.with_duouble_quote(|s| {
                s.write(&entrypoint.entrypoint);
            });
            s.write_symbol("]");
            s.write_line(None);

            s.write_token("rules");
            s.write_symbol(": ");
            s.write_symbol("\"tuple[");
            s.iter_and_join(&entrypoint.rules, " | ", |s, rule| {
                let rel_name = format!(
                    "{}{}Rule",
                    &codegen_utils::camel_case(&entrypoint.entrypoint),
                    &codegen_utils::camel_case(&rule.data.relationship),
                );

                s.write(&rel_name);
                state.add_import(PythonImport::new_local(&entrypoint.entrypoint, &rel_name))
            });
            s.write_symbol(", ...]\"");
            s.write_line(None);
        });

        state.write_code(s.serialize());
        state.add_import(PythonImport::new_global("ruu_runtime", "Entrypoint"));
        state.add_import(PythonImport::new_global("ruu_runtime", "registry"));
        state.add_import(PythonImport::new_global("typing", "Literal"));

        Some(state)
    }

    fn serialize_footer(&self) -> Option<CodegenState<PythonImport>> {
        let mut s = self.new_codegen_helper();

        let entrypoints = &self.file.entrypoints;
        let fragments = &self.file.fragments;

        if entrypoints.len() == 0 && fragments.len() == 0 {
            return None;
        }

        let name = format!(
            "{}Schema",
            &codegen_utils::camel_case(self.file_name.file_stem().unwrap().to_str().unwrap())
        );

        s.write_line(Some("@registry.bind"));
        PythonImport::with_class(&mut s, &name, vec!["Schema"], |s| {
            if entrypoints.len() > 0 {
                s.write_token("entrypoints");
                s.write_symbol(": ");
                s.write_symbol("\"tuple[");
                s.iter_and_join(entrypoints, " | ", |s, entrypoint| {
                    let entrypoint_name = format!(
                        "{}Entrypoint",
                        &codegen_utils::camel_case(&entrypoint.data.entrypoint)
                    );

                    s.write(&entrypoint_name);
                });
                s.write_symbol(", ...]\"");
                s.write_line(None);
            }

            if fragments.len() > 0 {
                s.write_token("fragments");
                s.write_symbol(": ");
                s.write_symbol("\"tuple[");
                s.iter_and_join(fragments, " | ", |s, fragment| {
                    let fragment_name = format!(
                        "{}{}Fragment",
                        &codegen_utils::camel_case(&fragment.data.for_entity),
                        &codegen_utils::camel_case(&fragment.data.name)
                    );

                    s.write(&fragment_name);
                });
                s.write_symbol(", ...]\"");
                s.write_line(None);
            }

            s.write_line(None);
            s.write_line(Some("@classmethod"));
            s.write_line(Some("def load_from_obj(cls):"));
            s.with_indent(|s| {
                s.write_line(Some("assert cls._registry"));
                s.write_line(Some("cls._registry.update_annotations()"));
                s.write_line(None);
                s.write_line(Some("result = cls.parse_obj(json.loads(\"\"\""));
                s.with_indent(|s| {
                    let data = &self.file.clone();
                    let as_json = serde_json::to_string(&data).unwrap();
                    s.write_line(Some(&as_json));
                });
                s.write_line(Some("\"\"\"))"));
                s.write_line(None);
                s.write_line(Some("result.register_globals()"));
                s.write_line(Some("return result"));
            });
        });
        s.write_line(None);
        s.write_line(Some(format!("schema = {}.load_from_obj()", &name).as_str()));

        let mut state = CodegenState::new();
        state.write_code(s.serialize());
        state.add_import(PythonImport::new_global("ruu_runtime", "Schema"));
        state.add_import(PythonImport::new_global("ruu_runtime", "registry"));
        state.add_import(PythonImport::new_global_module("json"));

        Some(state)
    }
}
