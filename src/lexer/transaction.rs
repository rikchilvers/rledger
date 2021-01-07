use super::amount::Amount;
use super::posting::*;
use super::status::Status;
use super::transaction_header::*;
use nom::{combinator::map_res, multi::many1, sequence::tuple, IResult};

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Transaction {
    date: String,
    payee: String,
    status: Status,
    postings: Vec<Posting>,
}

impl Transaction {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_header_and_postings(lexed: (TransactionHeader, Vec<Posting>)) -> Self {
        Transaction {
            date: lexed.0.date,
            status: lexed.0.status,
            payee: lexed.0.payee,
            postings: lexed.1,
        }
    }
}

// pub fn transaction(i: &str) -> IResult<&str, Transaction> {
//     map_res::<_, _, _, _, nom::error::Error<&str>, _, _>(
//         tuple((transaction_header, many1(posting))),
//         |x: (TransactionHeader, Vec<Posting>)| Ok(Transaction::from_header_and_postings(x)),
//     )(i)
//     // let (i, th) = transaction_header(i)?;
//     // return Ok((i, Transaction::from_header(th)));
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let th = TransactionHeader {
            date: "2020-01-01".to_owned(),
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
        let expected = Transaction::from_header_and_postings((th, ps));
        let input = "2020-01-01 * A Shop\n\tAssets:Current  £15\n";
        // let t = transaction(input);
        // assert_eq!(transaction(input), Ok(((""), expected)));
    }
}
