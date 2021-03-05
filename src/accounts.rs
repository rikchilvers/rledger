use journal::Posting;
use reader::error::Error;
use reader::reader::{Config, Reader};
use tree::Tree;

pub struct Accounts<'a> {
    tree: Tree<'a, usize>,
    postings: Vec<Posting>,
}

impl<'a> Accounts<'a> {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            postings: Vec::new(),
        }
    }

    pub fn read(&'a mut self, file: String) -> Result<(), Error> {
        let mut reader = Reader::new();
        let config = Config::new();

        let (_, postings, _) = reader.read(file, config)?;
        self.postings = postings;

        for posting in &self.postings {
            let mut path: Vec<&str> = posting.path.split(':').collect();
            self.tree.add_path(&mut path);
        }

        self.tree.display(&None, |_| None);

        Ok(())
    }
}
