// Taken from https://stackoverflow.com/a/45882510

use std::{
    fs::File,
    io::{self, prelude::*},
    sync::Arc,
};

pub struct BufReader {
    reader: io::BufReader<File>,
    buffer: Arc<String>,
}

fn new_buffer() -> Arc<String> {
    Arc::new(String::with_capacity(1024)) // Tweakable capacity
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
    type Item = io::Result<Arc<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = match Arc::get_mut(&mut self.buffer) {
            Some(buffer) => {
                buffer.clear();
                buffer
            }
            None => {
                self.buffer = new_buffer();
                Arc::make_mut(&mut self.buffer)
            }
        };

        self.reader
            .read_line(buffer)
            .map(|u| {
                if u == 0 {
                    return None;
                }

                if let Some(r) = Arc::get_mut(&mut self.buffer) {
                    // drop the newline
                    if r.as_bytes().last() == Some(&10) {
                        r.pop();
                    }
                } else {
                    panic!("couldn't take reader buffer as mutable");
                }

                return Some(Arc::clone(&self.buffer));
            })
            .transpose()
    }
}
