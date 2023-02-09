use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub relationship: String,
    pub attributes: Vec<Attribute>,
    pub grants: Vec<Vec<String>>,
    pub rules: Vec<Rule>,
    pub recursive: bool,
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.relationship.hash(state);
        self.attributes.hash(state);
        self.grants.hash(state);
        self.rules.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrypoint {
    pub entrypoint: String,
    pub rules: Vec<Rule>,
}

impl Hash for Entrypoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entrypoint.hash(state);
        self.rules.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlangFile {
    pub entrypoints: Vec<Entrypoint>,
}

impl Hash for SlangFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entrypoints.hash(state);
    }
}

pub trait SlangSerialize {
    fn slang_serialize(&self, indent: usize) -> String;
}

impl SlangSerialize for Attribute {
    fn slang_serialize(&self, _indent: usize) -> String {
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

impl SlangSerialize for Rule {
    fn slang_serialize(&self, indent: usize) -> String {
        if self.relationship == "*" {
            return format!("{}*\n", " ".repeat(indent * 4));
        }

        let mut result = String::new();

        result.push_str(format!("{}{}", " ".repeat(indent * 4), self.relationship).as_str());

        if self.attributes.len() > 0 {
            for attr in self.attributes.iter() {
                result.push_str(attr.slang_serialize(indent + 1).as_str());
            }
        }

        result.push_str(" {\n");

        if self.grants.len() > 0 {
            for grant in self.grants.iter() {
                let grant_str = grant.join(".");
                result.push_str(
                    format!("{}{};\n", " ".repeat((indent + 1) * 4).as_str(), grant_str).as_str(),
                );
            }

            if self.rules.len() > 0 {
                result.push_str("\n");
            }
        }

        for (i, rule) in self.rules.iter().enumerate() {
            result.push_str(rule.slang_serialize(indent + 1).as_str());

            if i < self.rules.len() - 1 {
                result.push_str("\n");
            }
        }

        result.push_str(format!("{}}}\n", " ".repeat(indent * 4)).as_str());
        result
    }
}

impl SlangSerialize for Entrypoint {
    fn slang_serialize(&self, indent: usize) -> String {
        let mut result = String::new();

        result.push_str(format!("{}@{} {{\n", " ".repeat(indent * 4), self.entrypoint).as_str());

        for (i, rule) in self.rules.iter().enumerate() {
            result.push_str(rule.slang_serialize(indent + 1).as_str());

            if i < self.rules.len() - 1 {
                result.push_str("\n");
            }
        }

        result.push_str("}");
        result
    }
}

impl SlangSerialize for SlangFile {
    fn slang_serialize(&self, indent: usize) -> String {
        let mut result = String::new();

        for (i, entrypoint) in self.entrypoints.iter().enumerate() {
            result.push_str(entrypoint.slang_serialize(indent).as_str());
            result.push_str("\n");

            if i < self.entrypoints.len() - 1 {
                result.push_str("\n");
            }
        }

        result
    }
}
