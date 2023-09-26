use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Eq)]
pub struct Parsed<T> {
    pub loc: Option<(usize, usize)>,
    pub file_name: Option<PathBuf>,
    pub docstring: Option<String>,
    pub data: T,
}

impl<T> Parsed<T> {
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

    pub fn as_with_data<U>(&self, new_data: U) -> Parsed<U> {
        Parsed {
            loc: self.loc,
            file_name: self.file_name.clone(),
            docstring: self.docstring.clone(),
            data: new_data,
        }
    }

    pub fn into_with_data<U>(self, new_data: U) -> (Parsed<U>, T) {
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

impl<T> Hash for Parsed<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T> Serialize for Parsed<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}
