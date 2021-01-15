extern crate clap;

mod command;
mod journal;
mod print;

use crate::command::Command;
use crate::journal::Reader;
use crate::print::Printer;

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
        // let printer = Printer::new();
        let reader = Reader::new(matches.value_of("file").unwrap());

        for transaction in reader {
            println!("{}", transaction.borrow())
        }

        // printer.report_take(
        //     reader
        //         .read(matches.value_of("file").unwrap())
        //         .unwrap_or(&vec![]),
        // );
        // drop(reader); // this is necessary but perhaps it means there's a better way?
    }
}
