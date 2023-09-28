use serde::{Deserialize, Serialize};

use super::{
    parse_location::{Context, Descendable, DescendableChildren, Parsed},
    ruulang_ast::Attribute,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuuLangSchema {
    pub entities: Vec<Parsed<Entity>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub name: Parsed<String>,
    pub relationships: Vec<Parsed<Relationship>>,
    pub grants: Vec<Parsed<Vec<String>>>,
}

impl<'a> DescendableChildren<'a> for Entity {
    fn context_and_name(&self) -> (Context<'a>, Option<String>) {
        (Context::None, Some(self.name.data.clone()))
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_name: Parsed<String>,
    pub entity_name: Parsed<String>,
    pub attributes: Vec<Parsed<Attribute>>,
}

impl<'a> DescendableChildren<'a> for Relationship {
    fn context_and_name(&self) -> (Context<'a>, Option<String>) {
        (Context::None, Some(self.relationship_name.data.clone()))
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
