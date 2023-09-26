use std::collections::HashMap;

use crate::{
    parser::{parse_location::Parsed, schema_ast::Relationship},
    utils::trie::Trie,
};

#[derive(Debug, Clone)]
pub struct TcEntity {
    pub name: String,

    relationships: HashMap<String, Parsed<Relationship>>,
    grants: Trie<String, Parsed<()>>,
}

impl TcEntity {
    pub fn new(name: String) -> Self {
        return Self {
            name,
            relationships: HashMap::new(),
            grants: Trie::new(),
        };
    }

    pub fn add_relationship(&mut self, rel: Parsed<Relationship>) -> bool {
        if self.relationships.contains_key(&rel.data.relationship_name) {
            return false;
        }

        let rels = &mut self.relationships;
        rels.insert(rel.data.relationship_name.clone(), rel);

        true
    }

    pub fn add_grant(&mut self, grant: Parsed<Vec<String>>) {
        if self.grants.contains(&grant.data) {
            return;
        }

        let (parsed_context, data) = grant.into_with_data(());
        self.grants.add(&data, parsed_context);
    }

    pub fn get_rule(&self, rule: &String) -> Option<&Parsed<Relationship>> {
        self.relationships.get(rule)
    }

    pub fn allows_grant(&self, grant: &Vec<String>) -> bool {
        self.grants.contains_prefix(grant)
    }
}
