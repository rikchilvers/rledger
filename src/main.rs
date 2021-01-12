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
                .default_value("$LEDGER_FILE")
                .takes_value(true)
                .value_name("LEDGER_FILE"),
        )
        .subcommand(
            App::new("print")
                .about("Show transaction entries.")
                .alias("p"),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("print") {
        let path = "tests/test.journal";
        // let path = "/Users/rik/Documents/Personal/Finance/current.journal";
        //
        let mut printer = Printer::new();
        let mut reader = Reader::new(Box::new(|| printer.handle_posting()));

        reader.read(path);
    }
}
