extern crate clap;

mod command;
mod journal;
mod reader;

use crate::command::Command;
use crate::command::Printer;
use crate::reader::Reader;

use clap::{App, Arg};

fn main() {
    let matches = App::new("rledger")
        .author("Rik Chilvers <rikchilvers@fastmail.com>")
        .version("0.1.0")
        .about("A reimplementation of ledger with YNAB-style budgeting at its core.")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("The journal file to read.")
                .env("LEDGER_FILE")
                .value_name("LEDGER_FILE"),
        )
        .subcommand(
            App::new("print")
                .about("Show transaction entries.")
                .alias("p"),
        )
        .get_matches();

    if matches.value_of("file").is_none() && matches.occurrences_of("file") == 0 {
        println!("No journal file was passed and none could be found in the environment.");
        return;
    }

    if let Some(_) = matches.subcommand_matches("print") {
        let mut printer = Printer::new();
        let mut reader = Reader::new(Box::new(|t| printer.handle_transaction(t)));

        reader.read(matches.value_of("file").unwrap());
        drop(reader); // this is necessary but perhaps it means there's a better way?

        printer.report();
    }
}
