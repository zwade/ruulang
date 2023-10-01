use serde::Serialize;

use super::{
    parse_location::{Context, Descendable, DescendableChildren, Identifier, Parsed},
    ruulang_ast::{Attribute, Grant},
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuuLangSchema {
    pub entities: Vec<Parsed<Entity>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entity {
    pub name: Parsed<Identifier>,
    pub relationships: Vec<Parsed<Relationship>>,
    pub grants: Vec<Parsed<Grant>>,
}

impl<'a> DescendableChildren<'a> for Entity {
    fn context_and_name(&'a self) -> (Context<'a>, Option<String>) {
        (Context::Entity(&self), Some(self.name.data.value.clone()))
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        self.relationships
            .iter()
            .map(|x| x as &dyn Descendable)
            .chain(self.grants.iter().map(|x| x as &dyn Descendable))
            .chain(std::iter::once(&self.name as &dyn Descendable))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Relationship {
    pub relationship_name: Parsed<Identifier>,
    pub entity_name: Parsed<Identifier>,
    pub attributes: Vec<Parsed<Attribute>>,
}

impl<'a> DescendableChildren<'a> for Relationship {
    fn context_and_name(&'a self) -> (Context<'a>, Option<String>) {
        (
            Context::Relationship(&self),
            Some(self.relationship_name.data.value.clone()),
        )
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        self.attributes
            .iter()
            .map(|x| x as &dyn Descendable)
            .chain(std::iter::once(&self.relationship_name as &dyn Descendable))
            .chain(std::iter::once(&self.entity_name as &dyn Descendable))
            .collect()
    }
}
