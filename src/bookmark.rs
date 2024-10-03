use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct BookmarkStore {
    bookmarks: BTreeMap<String, usize>,
}

impl BookmarkStore {
    pub fn new_bookmark(&mut self, name: &str, offset: usize) {
        self.bookmarks.insert(name.to_string(), offset);
    }
}
