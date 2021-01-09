use super::{posting::*, status::Status, transaction_header::*};

#[derive(Debug, Eq, PartialEq)]
pub struct Transaction {
    pub date: time::Date,
    pub payee: String,
    pub status: Status,
    pub postings: Vec<Posting>,
}

impl Transaction {
    pub fn from_header(header: TransactionHeader) -> Self {
        Transaction {
            date: header.date,
            status: header.status,
            payee: header.payee,
            postings: vec![],
        }
    }

    pub fn add_posting(&mut self, posting: Posting) {
        self.postings.push(posting);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::amount::Amount;

    #[test]
    fn it_works() {
        let th = TransactionHeader {
            date: time::Date::try_from_ymd(2020, 01, 01).unwrap(),
            payee: "A Shop".to_owned(),
            status: Status::Cleared,
            comment: None,
        };
        let ps = vec![Posting {
            path: "Assets:Current".to_owned(),
            amount: Some(Amount {
                commodity: "£".to_owned(),
                quantity: 1500,
            }),
        }];
        // let expected = Transaction::from_header_and_postings((th, ps));
        // let input = "2020-01-01 * A Shop\n\tAssets:Current  £15\n";
        // let t = transaction(input);
        // assert_eq!(transaction(input), Ok(((""), expected)));
    }
}
