use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use super::parse_location::Parsed;

pub trait RuuLangSerialize {
    fn ruulang_serialize(&self, indent: usize) -> String;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub arguments: Vec<String>,
}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.hash(state);
    }
}

impl RuuLangSerialize for Attribute {
    fn ruulang_serialize(&self, _indent: usize) -> String {
        let mut result = String::new();

        result.push_str(format!(":{}", self.name).as_str());

        if self.arguments.len() > 0 {
            result.push_str("(");

            for (i, arg) in self.arguments.iter().enumerate() {
                result.push_str(arg);

                if i < self.arguments.len() - 1 {
                    result.push_str(", ");
                }
            }

            result.push_str(")");
        }

        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub relationship: String,
    pub attributes: Vec<Parsed<Attribute>>,
    pub grants: Vec<Parsed<Vec<String>>>,
    pub rules: Vec<Parsed<Rule>>,
    pub recursive: bool,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub include_fragments: Vec<Parsed<String>>,
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.relationship.hash(state);
        self.attributes.hash(state);
        self.include_fragments.hash(state);
        self.grants.hash(state);
        self.rules.hash(state);
        self.recursive.hash(state);
    }
}

impl RuuLangSerialize for Rule {
    fn ruulang_serialize(&self, indent: usize) -> String {
        if self.relationship == "*" {
            return format!("{}*\n", " ".repeat(indent * 4));
        }

        let mut result = String::new();

        result.push_str(format!("{}{}", " ".repeat(indent * 4), self.relationship).as_str());

        if self.attributes.len() > 0 {
            for attr in self.attributes.iter() {
                result.push_str(attr.data.ruulang_serialize(indent + 1).as_str());
            }
        }

        result.push_str(" {");

        if self.grants.len() > 0 {
            result.push_str("\n");

            for grant in self.grants.iter() {
                let grant_str = grant.data.join(".");
                result.push_str(
                    format!("{}{};\n", " ".repeat((indent + 1) * 4).as_str(), grant_str).as_str(),
                );
            }
        }

        if self.include_fragments.len() > 0 {
            result.push_str("\n");

            for fragment in self.include_fragments.iter() {
                result.push_str(
                    format!(
                        "{}#{};\n",
                        " ".repeat((indent + 1) * 4).as_str(),
                        fragment.data
                    )
                    .as_str(),
                );
            }
        }

        if self.rules.len() > 0 {
            result.push_str("\n");

            for (i, rule) in self.rules.iter().enumerate() {
                result.push_str(rule.data.ruulang_serialize(indent + 1).as_str());

                if i < self.rules.len() - 1 {
                    result.push_str("\n");
                }
            }
        }

        result.push_str(format!("{}}}\n", " ".repeat(indent * 4)).as_str());
        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrypoint {
    pub entrypoint: String,
    pub rules: Vec<Parsed<Rule>>,
}

impl Hash for Entrypoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entrypoint.hash(state);
        self.rules.hash(state);
    }
}

impl RuuLangSerialize for Entrypoint {
    fn ruulang_serialize(&self, indent: usize) -> String {
        let mut result = String::new();

        result.push_str(format!("{}@{} {{\n", " ".repeat(indent * 4), self.entrypoint).as_str());

        for (i, rule) in self.rules.iter().enumerate() {
            result.push_str(rule.data.ruulang_serialize(indent + 1).as_str());

            if i < self.rules.len() - 1 {
                result.push_str("\n");
            }
        }

        result.push_str("}");
        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fragment {
    pub name: String,
    pub for_entity: String,
    pub rules: Vec<Parsed<Rule>>,
    pub grants: Vec<Parsed<Vec<String>>>,
}

impl Hash for Fragment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.rules.hash(state);
    }
}

impl RuuLangSerialize for Fragment {
    fn ruulang_serialize(&self, indent: usize) -> String {
        let mut result = String::new();

        result.push_str(format!("{}fragment {} {{", " ".repeat(indent * 4), self.name).as_str());

        if self.grants.len() > 0 {
            result.push_str("\n");

            for grant in self.grants.iter() {
                let grant_str = grant.data.join(".");
                result.push_str(
                    format!("{}{};\n", " ".repeat((indent + 1) * 4).as_str(), grant_str).as_str(),
                );
            }
        }

        if self.rules.len() > 0 {
            result.push_str("\n");

            for (i, rule) in self.rules.iter().enumerate() {
                result.push_str(rule.data.ruulang_serialize(indent + 1).as_str());

                if i < self.rules.len() - 1 {
                    result.push_str("\n");
                }
            }
        }

        result.push_str(format!("{}}}\n", " ".repeat(indent * 4)).as_str());
        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuuLangFile {
    pub entrypoints: Vec<Parsed<Entrypoint>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fragments: Vec<Parsed<Fragment>>,
}

impl Hash for RuuLangFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entrypoints.hash(state);
    }
}

impl RuuLangSerialize for RuuLangFile {
    fn ruulang_serialize(&self, indent: usize) -> String {
        let mut result = String::new();

        for fragment in self.fragments.iter() {
            result.push_str(fragment.data.ruulang_serialize(indent).as_str());
            result.push_str("\n");
        }

        if self.fragments.len() > 0 && self.entrypoints.len() > 0 {
            result.push_str("\n");
        }

        for (i, entrypoint) in self.entrypoints.iter().enumerate() {
            result.push_str(entrypoint.data.ruulang_serialize(indent).as_str());
            result.push_str("\n");

            if i < self.entrypoints.len() - 1 {
                result.push_str("\n");
            }
        }

        result
    }
}
