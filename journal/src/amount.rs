#[derive(Debug, Default, Eq, PartialEq)]
pub struct Amount {
    pub commodity: String,
    pub quantity: i64,
}

impl Amount {
    pub fn new(quantity: i64, commodity: &str) -> Self {
        Amount {
            quantity,
            // TODO: have new take a String
            commodity: commodity.to_owned(),
        }
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quantity = self.quantity as f64 / 100.;
        write!(f, "{}{:.2}", self.commodity, quantity)
    }
}
