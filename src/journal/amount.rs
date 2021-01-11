#[derive(Debug, Default, Eq, PartialEq)]
pub struct Amount {
    pub commodity: String,
    pub quantity: i64,
}

impl Amount {
    pub fn new(quantity: i64, commodity: &str) -> Self {
        Amount {
            quantity,
            commodity: commodity.to_owned(),
        }
    }
}
