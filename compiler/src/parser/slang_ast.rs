use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use pyo3::{pyclass, pymethods, types::PyDict, Python, ToPyObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct Attribute {
    pub name: String,
    pub arguments: Vec<String>,
}

#[pymethods]
impl Attribute {
    #[new]
    fn new(name: String, arguments: Vec<String>) -> Self {
        Attribute { name, arguments }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "Attribute(name={}, arguments={:?})",
            self.name, self.arguments
        )
    }

    fn serialize(&self) -> String {
        self.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        let _ = dict.set_item("name", self.name.to_object(py));
        let _ = dict.set_item("arguments", self.arguments.to_object(py));
        dict
    }
}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct Rule {
    pub relationship: String,
    pub attributes: Vec<Attribute>,
    pub grants: Vec<Vec<String>>,
    pub rules: Vec<Rule>,
    pub recursive: bool,
}

#[pymethods]
impl Rule {
    #[new]
    fn new(
        relationship: String,
        attributes: Vec<Attribute>,
        grants: Vec<Vec<String>>,
        rules: Vec<Rule>,
        recursive: bool,
    ) -> Self {
        Rule {
            relationship,
            attributes,
            grants,
            rules,
            recursive,
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "Relationship(relationship={}, attributes={:?}, grants={:?}, rules={:?})",
            self.relationship, self.attributes, self.grants, self.rules
        )
    }

    fn serialize(&self) -> String {
        self.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        let _ = dict.set_item("relationship", self.relationship.to_object(py));
        let _ = dict.set_item(
            "attributes",
            self.attributes
                .iter()
                .map(|attr| attr.dict(py))
                .collect::<Vec<_>>()
                .to_object(py),
        );
        let _ = dict.set_item("grants", self.grants.to_object(py));
        let _ = dict.set_item(
            "rules",
            self.rules
                .iter()
                .map(|rule| rule.dict(py))
                .collect::<Vec<_>>()
                .to_object(py),
        );
        dict
    }
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
#[pyclass]
pub struct Entrypoint {
    pub entrypoint: String,
    pub rules: Vec<Rule>,
}

#[pymethods]
impl Entrypoint {
    #[new]
    fn new(entrypoint: String, rules: Vec<Rule>) -> Self {
        Entrypoint { entrypoint, rules }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "Entrypoint(entrypoint={}, rules={:?})",
            self.entrypoint, self.rules
        )
    }

    fn serialize(&self) -> String {
        self.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        let _ = dict.set_item("entrypoint", self.entrypoint.to_object(py));
        let _ = dict.set_item(
            "rules",
            self.rules
                .iter()
                .map(|rule| rule.dict(py))
                .collect::<Vec<_>>()
                .to_object(py),
        );
        dict
    }
}

impl Hash for Entrypoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entrypoint.hash(state);
        self.rules.hash(state);
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

impl SlangSerialize for Vec<Entrypoint> {
    fn slang_serialize(&self, indent: usize) -> String {
        let mut result = String::new();

        for (i, entrypoint) in self.iter().enumerate() {
            result.push_str(entrypoint.slang_serialize(indent).as_str());
            result.push_str("\n");

            if i < self.len() - 1 {
                result.push_str("\n");
            }
        }

        result
    }
}
