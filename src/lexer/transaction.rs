use super::status::Status;
use super::transaction_header::*;
use nom::IResult;

#[derive(Default, Debug)]
pub struct Transaction {
    date: String,
    payee: String,
    status: Status,
}

impl Transaction {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_header(header: TransactionHeader) -> Self {
        Transaction {
            date: header.0,
            status: header.1,
            payee: header.2,
            ..Default::default()
        }
    }
}

pub fn transaction(i: &str) -> IResult<&str, Transaction> {
    let (i, th) = transaction_header(i)?;
    return Ok((i, Transaction::from_header(th)));
}
