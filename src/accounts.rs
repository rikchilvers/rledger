use reader::error::Error;
use reader::reader::{Config, Reader};
use tree::Tree;

pub struct Accounts<'a> {
    tree: Tree<'a, usize>,
}

impl<'a> Accounts<'a> {
    pub fn new() -> Self {
        Self { tree: Tree::new() }
    }

    pub fn read(&mut self, file: String) -> Result<(), Error> {
        let mut reader = Reader::new();
        let mut config = Config::new();
        config.should_sort = true;

        let (transactions, postings, _) = reader.read(file, config)?;

        for transaction in transactions {
            transaction.display(&postings);
        }

        Ok(())
    }
}
