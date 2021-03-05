use journal::Amount;
use journal::Posting;
use reader::error::Error;
use reader::reader::{Config, Reader};
use tree::Tree;

struct Account {
    amount: Amount,
}

impl Default for Account {
    fn default() -> Account {
        Account {
            amount: Amount::new(0, ""),
        }
    }
}

pub struct Balance<'a> {
    tree: Tree<'a, Account>,
    postings: Vec<Posting>,
}

impl<'a> Balance<'a> {
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
            let index = self.tree.add_path(&mut path);

            self.tree.walk_ancestors(index, |node| {
                node.value.amount.quantity += posting.amount.as_ref().unwrap().quantity
            });
        }

        self.tree.display(&None, |node| {
            let s = format!("{}", node.value.amount.quantity);
            Some(s)
        });

        Ok(())
    }
}
