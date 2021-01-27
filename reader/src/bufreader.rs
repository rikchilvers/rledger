// Taken from https://stackoverflow.com/a/45882510

use std::{
    fs::File,
    io::{self, prelude::*},
    rc::Rc,
};

pub struct BufReader {
    reader: io::BufReader<File>,
    buffer: Rc<String>,
}

fn new_buffer() -> Rc<String> {
    Rc::new(String::with_capacity(1024)) // Tweakable capacity
}

impl BufReader {
    pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let buffer = new_buffer();

        Ok(Self { reader, buffer })
    }
}

impl Iterator for BufReader {
    type Item = io::Result<Rc<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = match Rc::get_mut(&mut self.buffer) {
            Some(buffer) => {
                buffer.clear();
                buffer
            }
            None => {
                self.buffer = new_buffer();
                Rc::make_mut(&mut self.buffer)
            }
        };

        self.reader
            .read_line(buffer)
            .map(|u| if u == 0 { None } else { Some(Rc::clone(&self.buffer)) })
            .transpose()
    }
}
