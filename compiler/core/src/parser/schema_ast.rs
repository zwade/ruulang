use serde::{Deserialize, Serialize};

use super::{parse_location::Parsed, ruulang_ast::Attribute};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuuLangSchema {
    pub entities: Vec<Parsed<Entity>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub relationships: Vec<Parsed<Relationship>>,
    pub grants: Vec<Parsed<Vec<String>>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_name: String,
    pub entity_name: String,
    pub attributes: Vec<Parsed<Attribute>>,
}
