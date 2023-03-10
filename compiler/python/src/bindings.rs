use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use pyo3::{pyclass, pymethods, types::PyDict, Python};
use serde::{Deserialize, Serialize};
use slang_core::parser::slang_ast::{Attribute, Entrypoint, Rule, SlangFile, SlangSerialize, Fragment};

trait AsDict {
    fn as_dict<'a>(&self, py: Python<'a>) -> &'a PyDict;
}

impl AsDict for Attribute {
    fn as_dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name.clone()).unwrap();
        dict.set_item("arguments", self.arguments.clone()).unwrap();
        dict
    }
}

impl AsDict for Rule {
    fn as_dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        dict.set_item("relationship", self.relationship.clone())
            .unwrap();
        dict.set_item(
            "attributes",
            self.attributes
                .iter()
                .map(|attr| attr.as_dict(py))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        dict.set_item("grants", self.grants.clone()).unwrap();
        dict.set_item(
            "rules",
            self.rules
                .iter()
                .map(|rule| rule.as_dict(py))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        dict.set_item("recursive", self.recursive).unwrap();
        dict
    }
}

impl AsDict for Entrypoint {
    fn as_dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        dict.set_item("entrypoint", self.entrypoint.clone())
            .unwrap();
        dict.set_item(
            "rules",
            self.rules
                .iter()
                .map(|rule| rule.as_dict(py))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        dict
    }
}

impl AsDict for SlangFile {
    fn as_dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        dict.set_item(
            "entrypoints",
            self.entrypoints
                .iter()
                .map(|entrypoint| entrypoint.as_dict(py))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        dict
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct PyAttribute {
    attribute: Attribute,
}

#[pymethods]
impl PyAttribute {
    #[new]
    fn new(name: String, arguments: Vec<String>) -> Self {
        Self {
            attribute: Attribute { name, arguments },
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.attribute.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "Attribute(name={}, arguments={:?})",
            self.attribute.name, self.attribute.arguments
        )
    }

    fn serialize(&self) -> String {
        self.attribute.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        self.attribute.as_dict(py)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct PyRule {
    rule: Rule,
}

#[pymethods]
impl PyRule {
    #[new]
    #[args(include_fragments="Vec::new()")]
    fn new(
        relationship: String,
        attributes: Vec<PyAttribute>,
        grants: Vec<Vec<String>>,
        rules: Vec<PyRule>,
        recursive: bool,
        include_fragments: Vec<String>,
    ) -> Self {
        Self {
            rule: Rule {
                relationship,
                attributes: attributes
                    .iter()
                    .map(|attr| attr.attribute.clone())
                    .collect::<Vec<_>>(),
                grants,
                rules: rules
                    .iter()
                    .map(|rule| rule.rule.clone())
                    .collect::<Vec<_>>(),
                recursive,
                include_fragments,
            },
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.rule.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "Relationship(relationship={}, attributes={:?}, grants={:?}, rules={:?})",
            self.rule.relationship, self.rule.attributes, self.rule.grants, self.rule.rules
        )
    }

    fn serialize(&self) -> String {
        self.rule.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        self.rule.as_dict(py)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct PyEntrypoint {
    entrypoint: Entrypoint,
}

#[pymethods]
impl PyEntrypoint {
    #[new]
    fn new(entrypoint: String, rules: Vec<PyRule>) -> Self {
        Self {
            entrypoint: Entrypoint {
                entrypoint,
                rules: rules.iter().map(|r| r.rule.clone()).collect::<Vec<_>>(),
            },
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.entrypoint.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "Entrypoint(entrypoint={}, rules={:?})",
            self.entrypoint.entrypoint, self.entrypoint.rules
        )
    }

    fn serialize(&self) -> String {
        self.entrypoint.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        self.entrypoint.as_dict(py)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct PyFragment {
    fragment: Fragment,
}

#[pymethods]
impl PyFragment {
    #[new]
    fn new(name: String, grants: Vec<Vec<String>>, rules: Vec<PyRule>) -> Self {
        Self {
            fragment: Fragment {
                name,
                grants,
                rules: rules.iter().map(|r| r.rule.clone()).collect::<Vec<_>>(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct PySlangFile {
    slang_file: SlangFile,
}

#[pymethods]
impl PySlangFile {
    #[new]
    #[args(fragments="Vec::new()")]
    fn new(entrypoints: Vec<PyEntrypoint>, fragments: Vec<PyFragment>) -> Self {
        Self {
            slang_file: SlangFile {
                entrypoints: entrypoints
                    .iter()
                    .map(|e| e.entrypoint.clone())
                    .collect::<Vec<_>>(),

                fragments: fragments
                    .iter()
                    .map(|f| f.fragment.clone())
                    .collect::<Vec<_>>(),
            },
        }
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.slang_file.hash(&mut hasher);
        hasher.finish()
    }

    fn __repr__(&self) -> String {
        format!("SlangFile(entrypoints={:?})", self.slang_file.entrypoints)
    }

    fn serialize(&self) -> String {
        self.slang_file.slang_serialize(0)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn dict<'a>(&self, py: Python<'a>) -> &'a PyDict {
        self.slang_file.as_dict(py)
    }
}

impl PySlangFile {
    pub fn create(slang_file: SlangFile) -> Self {
        Self { slang_file }
    }
}
