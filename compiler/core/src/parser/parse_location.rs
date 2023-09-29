use std::{
    borrow::Borrow,
    fmt::Display,
    hash::{Hash, Hasher},
    ops::Deref,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use super::{
    ruulang_ast::{Entrypoint, Fragment, Rule},
    schema_ast::Entity,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Context<'a> {
    None,
    Rule(Box<&'a Rule>),
    Entrypoint(Box<&'a Entrypoint>),
    Entity(Box<&'a Entity>),
    Fragment(Box<&'a Fragment>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DescentContext<'a> {
    pub context: Context<'a>,
    pub name: Option<String>,

    pub docstring: &'a Option<String>,
}

impl<'a> DescentContext<'a> {
    pub fn new(context: Context<'a>, name: Option<String>, docstring: &'a Option<String>) -> Self {
        Self {
            context,
            name,
            docstring,
        }
    }
}

pub trait Descendable {
    fn descend_at(&self, loc: (usize, usize)) -> Option<Vec<DescentContext>>;
}

pub trait DescendableChildren<'a> {
    fn descend(&'a self) -> Vec<&dyn Descendable>;
    fn context_and_name(&'a self) -> (Context<'a>, Option<String>);
}

impl<'a> DescendableChildren<'a> for String {
    fn context_and_name(&self) -> (Context<'a>, Option<String>) {
        (Context::None, Some(self.clone()))
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        vec![]
    }
}

// Hack for grants
impl<'a> DescendableChildren<'a> for Vec<String> {
    fn context_and_name(&self) -> (Context<'a>, Option<String>) {
        (Context::None, Some(self.join(".")))
    }

    fn descend(&self) -> Vec<&dyn Descendable> {
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Eq)]
pub struct Parsed<T>
where
    for<'a> T: DescendableChildren<'a>,
{
    pub loc: Option<(usize, usize)>,
    pub file_name: Option<PathBuf>,
    pub docstring: Option<String>,
    pub data: T,
}

impl<T> Parsed<T>
where
    for<'a> T: DescendableChildren<'a>,
{
    pub fn new(
        data: T,
        loc: Option<(usize, usize)>,
        file_name: Option<PathBuf>,
        docstring: Option<String>,
    ) -> Self {
        Self {
            loc,
            file_name,
            docstring,
            data,
        }
    }

    pub fn new_at_loc(loc: (usize, usize), data: T) -> Self {
        Self {
            loc: Some(loc),
            file_name: None,
            docstring: None,
            data,
        }
    }

    pub fn as_with_data<U: for<'a> DescendableChildren<'a>>(&self, new_data: U) -> Parsed<U> {
        Parsed {
            loc: self.loc,
            file_name: self.file_name.clone(),
            docstring: self.docstring.clone(),
            data: new_data,
        }
    }

    pub fn into_with_data<U: for<'a> DescendableChildren<'a>>(self, new_data: U) -> (Parsed<U>, T) {
        let old_data = self.data;
        let new_parsed = Parsed {
            loc: self.loc,
            file_name: self.file_name,
            docstring: self.docstring,
            data: new_data,
        };

        (new_parsed, old_data)
    }

    pub fn into_with_filename(self, new_filename: PathBuf) -> (Parsed<T>, Option<PathBuf>) {
        let old_filename = self.file_name;
        let new_parsed = Parsed {
            loc: self.loc,
            file_name: Some(new_filename),
            docstring: self.docstring,
            data: self.data,
        };

        (new_parsed, old_filename)
    }

    pub fn into_with_docstring(self, new_docstring: Option<String>) -> (Parsed<T>, Option<String>) {
        let old_docstring = self.docstring;
        let new_parsed = Parsed {
            loc: self.loc,
            file_name: self.file_name,
            docstring: new_docstring,
            data: self.data,
        };

        (new_parsed, old_docstring)
    }
}

impl<T> Descendable for Parsed<T>
where
    T: for<'a> DescendableChildren<'a>,
{
    fn descend_at(&self, loc: (usize, usize)) -> Option<Vec<DescentContext>> {
        if let Some((start, end)) = self.loc {
            if loc.0 >= start && loc.0 <= end {
                let (context, name) = self.data.context_and_name();
                let mut result = vec![DescentContext::new(context, name, &self.docstring)];

                let children = self.data.descend();
                for child in children {
                    if let Some(mut ctx) = child.descend_at(loc) {
                        result.append(&mut ctx);
                        return Some(result);
                    }
                }

                return Some(result);
            }
        }

        None
    }
}

impl<T> Hash for Parsed<T>
where
    for<'a> T: Hash + DescendableChildren<'a>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T> Serialize for Parsed<T>
where
    for<'a> T: Serialize + DescendableChildren<'a>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<T> Display for Parsed<T>
where
    for<'a> T: Display + DescendableChildren<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<T> Deref for Parsed<T>
where
    for<'a> T: DescendableChildren<'a>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Borrow<T> for Parsed<T>
where
    for<'a> T: DescendableChildren<'a>,
{
    fn borrow(&self) -> &T {
        &self.data
    }
}
