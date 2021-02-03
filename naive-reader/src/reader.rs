use super::source::Source;

pub struct Reader {}

impl Reader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, location: &str) {
        let mut source = Source::new(location);
        source.lex();
    }
}
