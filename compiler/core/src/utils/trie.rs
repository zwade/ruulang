use core::hash::Hash;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trie<T, U>
where
    T: Hash + Eq + Clone,
    U: Hash + Eq + Clone,
{
    root: TrieNode<T, U>,
}

impl<T, U> Trie<T, U>
where
    T: Hash + Eq + Clone,
    U: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn add(&mut self, path: &Vec<T>, data: U) -> bool {
        self.root.add(path, data)
    }

    pub fn get(&self, path: &Vec<T>) -> Option<&U> {
        self.root.get(path)
    }

    pub fn contains(&self, path: &Vec<T>) -> bool {
        self.root.contains(path)
    }

    pub fn contains_prefix(&self, path: &Vec<T>) -> bool {
        self.root.contains_prefix(path)
    }

    pub fn contains_suffix(&self, path: &Vec<T>) -> bool {
        self.root.contains_suffix(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrieNode<T, U>
where
    T: Hash + Eq + Clone,
    U: Hash + Eq + Clone,
{
    children: HashMap<T, TrieNode<T, U>>,
    value: Option<U>,
}

impl<T, U> TrieNode<T, U>
where
    T: Hash + Eq + Clone,
    U: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            value: None,
        }
    }

    pub fn add(&mut self, path: &Vec<T>, data: U) -> bool {
        let head = path.first();
        let tail = path[1..].to_vec();

        if head.is_none() {
            return false;
        }

        let el = head.unwrap().clone();
        let child = self.children.entry(el).or_insert_with(|| TrieNode::new());

        if tail.is_empty() {
            if child.value.is_some() {
                return false;
            }

            child.value = Some(data);
            return true;
        } else {
            return child.add(&tail, data);
        }
    }

    pub fn get(&self, path: &Vec<T>) -> Option<&U> {
        let head = path.first();
        let tail = path[1..].to_vec();

        if head.is_none() {
            return None;
        }

        let el = head.unwrap();
        let child = self.children.get(el);

        if child.is_none() {
            return None;
        }

        let child = child.unwrap();

        if tail.is_empty() {
            return child.value.as_ref();
        } else {
            return child.get(&tail);
        }
    }

    pub fn contains(&self, path: &Vec<T>) -> bool {
        return self.contains_helper(path, false, false);
    }

    pub fn contains_prefix(&self, path: &Vec<T>) -> bool {
        return self.contains_helper(path, true, false);
    }

    pub fn contains_suffix(&self, path: &Vec<T>) -> bool {
        return self.contains_helper(path, false, true);
    }

    fn contains_helper(&self, path: &Vec<T>, allow_prefix: bool, allow_suffix: bool) -> bool {
        let head = path.first();
        let tail = path[1..].to_vec();

        if head.is_none() {
            return false;
        }

        let el = head.unwrap();
        let child = self.children.get(&el);

        if child.is_none() {
            return allow_suffix;
        }

        let child = child.unwrap();

        if tail.is_empty() {
            if child.value.is_none() {
                return allow_prefix;
            }

            return true;
        } else {
            return child.contains_helper(&tail, allow_prefix, allow_suffix);
        }
    }
}
