use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use super::{
    parse_location::{Context, Descendable, DescendableChildren, DescentContext, Parsed},
    schema_ast::Entity,
};

pub trait RuuLangSerialize {
    fn ruulang_serialize(&self, indent: usize) -> String;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grant {
    pub grant: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: Parsed<String>,
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

        result.push_str(format!(":{}", self.name.data).as_str());

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

impl<'a> DescendableChildren<'a> for Attribute {
    fn context_and_name(&self) -> (Context<'a>, Option<String>) {
        (Context::None, Some(self.name.data.clone()))
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        vec![&self.name as &dyn Descendable]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub relationship: Parsed<String>,
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
        if self.relationship.data == "*" {
            return format!("{}*\n", " ".repeat(indent * 4));
        }

        let mut result = String::new();

        result.push_str(format!("{}{}", " ".repeat(indent * 4), self.relationship.data).as_str());

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

impl<'a> DescendableChildren<'a> for Rule {
    fn context_and_name(&'a self) -> (Context<'a>, Option<String>) {
        (
            Context::Rule(Box::new(&self)),
            Some(self.relationship.data.clone()),
        )
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        self.attributes
            .iter()
            .map(|x| x as &dyn Descendable)
            .chain(self.grants.iter().map(|x| x as &dyn Descendable))
            .chain(self.rules.iter().map(|x| x as &dyn Descendable))
            .chain(self.include_fragments.iter().map(|x| x as &dyn Descendable))
            .chain(std::iter::once(&self.relationship as &dyn Descendable))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrypoint {
    pub entrypoint: Parsed<String>,
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

        result
            .push_str(format!("{}@{} {{\n", " ".repeat(indent * 4), self.entrypoint.data).as_str());

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

impl<'a> DescendableChildren<'a> for Entrypoint {
    fn context_and_name(&'a self) -> (Context<'a>, Option<String>) {
        (
            Context::Entrypoint(Box::new(&self)),
            Some(self.entrypoint.data.clone()),
        )
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        self.rules
            .iter()
            .map(|x| x as &dyn Descendable)
            .chain(std::iter::once(&self.entrypoint as &dyn Descendable))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fragment {
    pub name: Parsed<String>,
    pub for_entity: Parsed<String>,
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

        result
            .push_str(format!("{}fragment {} {{", " ".repeat(indent * 4), self.name.data).as_str());

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

impl<'a> DescendableChildren<'a> for Fragment {
    fn context_and_name(&'a self) -> (Context<'a>, Option<String>) {
        (
            Context::Fragment(Box::new(&self)),
            Some(self.name.data.clone()),
        )
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        self.rules
            .iter()
            .map(|x| x as &dyn Descendable)
            .chain(self.grants.iter().map(|x| x as &dyn Descendable))
            .chain(std::iter::once(&self.name as &dyn Descendable))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuuLangFile {
    pub entrypoints: Vec<Parsed<Entrypoint>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fragments: Vec<Parsed<Fragment>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub entities: Vec<Parsed<Entity>>,
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

impl<'a> DescendableChildren<'a> for RuuLangFile {
    fn context_and_name(&self) -> (Context<'a>, Option<String>) {
        (Context::None, None)
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        self.entrypoints
            .iter()
            .map(|x| x as &dyn Descendable)
            .chain(self.fragments.iter().map(|x| x as &dyn Descendable))
            .chain(self.entities.iter().map(|x| x as &dyn Descendable))
            .collect()
    }
}

impl Descendable for RuuLangFile {
    fn descend_at(
        &self,
        loc: (usize, usize),
    ) -> Option<Vec<super::parse_location::DescentContext>> {
        let children = self.descend();

        let mut result = vec![DescentContext::new(Context::None, None, &None)];
        for child in children {
            if let Some(mut ctx) = child.descend_at(loc) {
                result.append(&mut ctx);
                return Some(result);
            }
        }

        None
    }
}
