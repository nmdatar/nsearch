use std::collections::{VecDeque, HashSet};

pub struct Frontier {
    queue: VecDeque<String>,
    seen: HashSet<String>,
}

impl Frontier {

    pub fn new() -> Self {
        Frontier {
            queue: VecDeque::new(),
            seen: HashSet::new(),
        }
    }

    pub fn push(&mut self, url: String) {
        if !self.seen.contains(&url) {
            self.seen.insert(url.clone());
            self.queue.push_back(url);
        } 
    }

    pub fn pop(&mut self) -> Option<String> {
        self.queue.pop_front()
    }
}