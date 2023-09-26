use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WithOrigin<T> {
    pub origin: PathBuf,
    pub data: T,
}

impl<T> WithOrigin<T> {
    pub fn new(data: T, origin: PathBuf) -> Self {
        WithOrigin { data, origin }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> WithOrigin<U> {
        WithOrigin {
            data: op(self.data),
            origin: self.origin,
        }
    }

    pub fn as_with_data<U>(&self, data: U) -> WithOrigin<U> {
        WithOrigin {
            data,
            origin: self.origin.clone(),
        }
    }
}
