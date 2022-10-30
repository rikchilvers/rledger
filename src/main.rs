extern crate clap;
extern crate journal;
extern crate reader;
extern crate tree;

mod accounts;
mod balance;
mod budget;
mod command;
mod print;
mod stats;

use crate::accounts::Accounts;
use crate::balance::Balance;
use crate::budget::Budget;
use crate::print::Printer;
use crate::stats::Statistics;

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
        .subcommand(App::new("print").about("Show transaction entries").alias("p"))
        .subcommand(
            App::new("statistics")
                .about("Show statistics about the journal")
                .aliases(&["stats"]),
        )
        .subcommand(App::new("accounts").about("List all accounts").aliases(&["acc", "a"]))
        .subcommand(
            App::new("balance")
                .about("Show accounts and their balances")
                .aliases(&["bal"]),
        )
        .subcommand(App::new("budget").about("Show budget status").aliases(&["bud"]))
        .get_matches();

    if matches.value_of("file").is_none() && matches.occurrences_of("file") == 0 {
        println!("No journal file was passed and none could be found in the environment");
        return;
    }

    if let Some(_) = matches.subcommand_matches("print") {
        let file = matches.value_of("file").unwrap().to_owned();
        let mut printer = Printer::new();
        if let Err(e) = printer.read(file) {
            println!("{}", e);
        }
    }

    if let Some(_) = matches.subcommand_matches("accounts") {
        let file = matches.value_of("file").unwrap().to_owned();
        let mut accounts = Accounts::new();
        if let Err(e) = accounts.read(file) {
            println!("{}", e);
        }
    }

    if let Some(_) = matches.subcommand_matches("balance") {
        let file = matches.value_of("file").unwrap().to_owned();
        let mut balance = Balance::new();
        if let Err(e) = balance.read(file) {
            println!("{}", e);
        }
    }

    if let Some(_) = matches.subcommand_matches("budget") {
        let file = matches.value_of("file").unwrap().to_owned();
        let mut budget = Budget::new();
        if let Err(e) = budget.read(file) {
            println!("{}", e);
        }
    }

    if let Some(_) = matches.subcommand_matches("statistics") {
        let file = matches.value_of("file").unwrap().to_owned();
        let mut stats = Statistics::new();
        if let Err(e) = stats.read(file) {
            println!("{}", e);
        }
    }
}
