use std::collections::HashMap;

use crate::{
    parser::{
        parse_location::Parsed,
        schema_ast::Entity,
        slang_ast::{Entrypoint, Fragment, Rule, SlangFile},
    },
    utils::{
        error::{Result, SlangError, TypecheckError},
        with_origin::WithOrigin,
    },
};

use super::tc_ast::TcEntity;

#[derive(Debug, Clone)]
pub struct Typechecker<'a> {
    entities: HashMap<String, Box<TcEntity>>,
    fragments: HashMap<(String, String), &'a Parsed<Fragment>>,
}

impl<'a> Typechecker<'a> {
    pub fn new(
        entities: &'a Vec<WithOrigin<Parsed<Entity>>>,
        schemas: &'a Vec<WithOrigin<Result<SlangFile>>>,
    ) -> Self {
        let entity_map = Typechecker::parse_entities(entities, schemas);
        let fragments = Typechecker::parse_fragments(entities, schemas);

        Self {
            entities: entity_map,
            fragments,
        }
    }

    pub fn validate_file(&self, file: &SlangFile) -> Vec<SlangError> {
        let mut violations = vec![];

        for fragment in &file.fragments {
            let mut downstream_errors = self.validate_fragment(&fragment);
            violations.append(&mut downstream_errors);
        }

        for entrypoint in &file.entrypoints {
            let mut downstream_errors = self.validate_entrypoint(&entrypoint);
            violations.append(&mut downstream_errors);
        }

        violations
    }

    fn validate_entrypoint(&self, entrypoint: &Parsed<Entrypoint>) -> Vec<SlangError> {
        let mut violations = vec![];

        let starting_entity = match self.entities.get(&entrypoint.data.entrypoint) {
            None => {
                let missing_entity_name = &entrypoint.data.entrypoint;
                let missing_entity_error = entrypoint.as_with_data(format!(
                    "Unable to find entity name: {}",
                    missing_entity_name
                ));

                violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                    missing_entity_error,
                )));
                return violations;
            }

            Some(c) => c,
        };

        for rule in &entrypoint.data.rules {
            let mut downstream_errors = self.validate_rule(starting_entity, rule);
            violations.append(&mut downstream_errors);
        }

        violations
    }

    fn validate_fragment(&self, fragment: &Parsed<Fragment>) -> Vec<SlangError> {
        let mut violations = vec![];

        let starting_entity = match self.entities.get(&fragment.data.for_entity) {
            None => {
                let missing_entity_name = &fragment.data.for_entity;
                let missing_entity_error = fragment.as_with_data(format!(
                    "Unable to find entity name: {}",
                    missing_entity_name
                ));

                violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                    missing_entity_error,
                )));
                return violations;
            }

            Some(c) => c,
        };

        for grant in &fragment.data.grants {
            if !starting_entity.allows_grant(&grant.data) {
                let grant_str = grant.data.join(".");
                let grant_error = grant.as_with_data(format!(
                    "Entity {} does not allow grant: {}",
                    starting_entity.name, grant_str
                ));

                violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                    grant_error,
                )));
            }
        }

        for rule in &fragment.data.rules {
            let mut downstream_errors = self.validate_rule(starting_entity, rule);
            violations.append(&mut downstream_errors);
        }

        violations
    }

    fn validate_rule(
        &self,
        starting_entity: &TcEntity,
        current_rule: &Parsed<Rule>,
    ) -> Vec<SlangError> {
        let mut violations = vec![];

        let current_rel = match starting_entity.get_rule(&current_rule.data.relationship) {
            None => {
                if &current_rule.data.relationship == "*" {
                    return violations;
                }

                let missing_relationship_name = &current_rule.data.relationship;
                let bad_entity_name = &starting_entity.name;

                let missing_rule_error = current_rule.as_with_data(format!(
                    "Relationship {} not found for entity {}",
                    missing_relationship_name, bad_entity_name
                ));

                violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                    missing_rule_error,
                )));

                return violations;
            }

            Some(c) => c,
        };

        let current_entity = match self.entities.get(&current_rel.data.entity_name) {
            None => {
                let missing_entity_name = current_rel.data.entity_name.clone();
                let missing_entity_error = current_rel.as_with_data(format!(
                    "Unable to find entity name: {}",
                    missing_entity_name
                ));

                violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                    missing_entity_error,
                )));
                return violations;
            }

            Some(c) => c,
        };

        for grant in &current_rule.data.grants {
            if !current_entity.allows_grant(&grant.data) {
                let grant_name = grant.data.clone().join(".");

                let error = grant.as_with_data(format!("Invalid grant {}", grant_name));
                violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                    error,
                )));
            }
        }

        for included_fragment in &current_rule.data.include_fragments {
            let fragment_key = (included_fragment.data.clone(), current_entity.name.clone());

            match self.fragments.get(&fragment_key) {
                None => {
                    let missing_fragment_error = included_fragment.as_with_data(format!(
                        "Unable to find fragment name: {} for entity {}",
                        &fragment_key.0, &fragment_key.1
                    ));

                    violations.push(SlangError::TypecheckError(TypecheckError::GeneralError(
                        missing_fragment_error,
                    )));
                    continue;
                }
                Some(c) => c,
            };
        }

        for rule in &current_rule.data.rules {
            let mut downstream_errors = self.validate_rule(&current_entity, rule);
            violations.append(&mut downstream_errors);
        }

        violations
    }

    fn parse_entities(
        entities: &'a Vec<WithOrigin<Parsed<Entity>>>,
        schemas: &'a Vec<WithOrigin<Result<SlangFile>>>,
    ) -> HashMap<String, Box<TcEntity>> {
        let mut entity_map = HashMap::<String, Box<TcEntity>>::new();
        let schemas = schemas.iter().filter_map(|x| match &x.data {
            Ok(data) => {
                let new_schema = x.as_with_data(data);
                Some(new_schema)
            }
            Err(_) => None,
        });

        for origin in schemas {
            entities.iter().for_each(|entity| {
                let entity_name = &entity.data.data.name;
                let key = entity_name.clone();
                let name_for_insert = entity_name.clone();
                let found_entity = entity_map
                    .entry(key)
                    .or_insert_with(|| Box::new(TcEntity::new(name_for_insert)));

                entity.data.data.relationships.iter().for_each(|rel| {
                    let (updated_rel, _) = rel.clone().into_with_filename(origin.origin.clone());
                    found_entity.add_relationship(updated_rel);
                });

                entity.data.data.grants.iter().for_each(|grant| {
                    let (updated_grant, _) =
                        grant.clone().into_with_filename(origin.origin.clone());
                    found_entity.add_grant(updated_grant)
                });
            });
        }

        entity_map
    }

    fn parse_fragments(
        entities: &'a Vec<WithOrigin<Parsed<Entity>>>,
        schemas: &'a Vec<WithOrigin<Result<SlangFile>>>,
    ) -> HashMap<(String, String), &'a Parsed<Fragment>> {
        schemas
            .into_iter()
            .filter_map(|x| match &x.data {
                Ok(data) => {
                    let new_schema = x.as_with_data(data);
                    Some(new_schema)
                }
                Err(_) => None,
            })
            .flat_map(|schema| &schema.data.fragments)
            .map(|fragment| {
                let fragment_name = fragment.data.name.clone();
                let entity_name = fragment.data.for_entity.clone();
                ((fragment_name, entity_name), fragment)
            })
            .collect::<HashMap<_, _>>()
    }
}
