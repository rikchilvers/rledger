# rledger

Much like [zledger](https://github.com/rikchilvers/zledger) and [gledger](https://github.com/rikchilvers/gledger), the goal of rledger was to rewrite [ledger](https://github.com/ledger/ledger) in a language I was interested in while adding [YNAB](https://www.youneedabudget.com/)-style envelope budgeting.

The first version of rledger used [nom](https://github.com/Geal/nom), a parser combinator library. On paper, it seemed like this would make the process of writing a parser for the ledger file format but it turned out to be more challenging than I expected. In the end, I rewrote the parser to be more like gledger's which reads a ledger file line by line and treats the first character of each as a hint as to how to parse it.
