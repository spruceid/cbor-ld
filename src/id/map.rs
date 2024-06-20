use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct IdMap {
    parent: Option<&'static Self>,
    forward: HashMap<String, u64>,
    backward: HashMap<u64, String>,
}

impl IdMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_derived(parent: Option<&'static Self>) -> Self {
        Self {
            parent,
            forward: HashMap::new(),
            backward: HashMap::new(),
        }
    }

    pub fn get_id(&self, term: &str) -> Option<u64> {
        self.forward
            .get(term)
            .copied()
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_id(term)))
    }

    pub fn get_term(&self, id: u64) -> Option<&str> {
        self.backward
            .get(&id)
            .map(String::as_str)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_term(id)))
    }

    pub fn insert(&mut self, key: String, id: u64) {
        self.forward.insert(key.clone(), id);
        self.backward.insert(id, key);
    }
}

impl FromIterator<(&'static str, u64)> for IdMap {
    fn from_iter<T: IntoIterator<Item = (&'static str, u64)>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl Extend<(&'static str, u64)> for IdMap {
    fn extend<T: IntoIterator<Item = (&'static str, u64)>>(&mut self, iter: T) {
        for (term, id) in iter {
            self.insert(term.to_owned(), id);
        }
    }
}
