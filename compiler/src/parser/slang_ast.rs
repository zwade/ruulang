use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship: String,
    pub attributes: Vec<Attribute>,
    pub grants: Vec<String>,
    pub rules: Vec<Relationship>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrypoint {
    pub entrypoint: String,
    pub rules: Vec<Relationship>,
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

impl SlangSerialize for Relationship {
    fn slang_serialize(&self, indent: usize) -> String {
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
                result.push_str(format!("{}{};\n", " ".repeat((indent + 1) * 4).as_str(), grant).as_str());
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
