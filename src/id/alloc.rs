use crate::IdMap;

pub struct IdAllocator {
    term_to_id: IdMap,
    next_id: u64,
}

impl IdAllocator {
    pub fn new(term_to_id: Option<&'static IdMap>, next_id: u64) -> Self {
        Self {
            term_to_id: IdMap::new_derived(term_to_id),
            next_id,
        }
    }

    pub fn encode_term(&self, term: &str, plural: bool) -> Option<u64> {
        self.term_to_id
            .get_id(term)
            .map(|id| if plural { id + 1 } else { id })
    }

    pub fn decode_term(&self, id: u64) -> Option<(&str, bool)> {
        let singular = if id % 2 == 0 { id } else { id - 1 };

        self.term_to_id
            .get_term(singular)
            .map(|term| (term, singular != id))
    }

    pub fn allocate(&mut self, term: &str) -> u64 {
        match self.term_to_id.get_id(term) {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 2;
                self.term_to_id.insert(term.to_owned(), id);
                id
            }
        }
    }
}
