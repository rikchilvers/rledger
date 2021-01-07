use nom::{branch::alt, bytes::complete::take_until, combinator::rest, IResult};

// TODO: this needs to not accept comments
pub fn account(i: &str) -> IResult<&str, &str> {
    alt((
        take_until("  "),
        take_until("\t"),
        take_until(" ;"),
        take_until(" #"),
        take_until(";"),
        take_until("#"),
        rest,
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_accounts_before_tabs() {
        let input = "Assets:Savings:ISA\t";
        assert_eq!(account(input), Ok(("\t", "Assets:Savings:ISA")));
    }

    #[test]
    fn it_matches_accounts_before_spaces() {
        let input = "Assets:Savings:ISA  ";
        assert_eq!(account(input), Ok(("  ", "Assets:Savings:ISA")));
    }

    #[test]
    fn it_matches_accounts_before_comments() {
        let input = "Assets:Savings:ISA#";
        assert_eq!(account(input), Ok(("#", "Assets:Savings:ISA")));

        let input = "Assets:Savings:ISA ;";
        assert_eq!(account(input), Ok((" ;", "Assets:Savings:ISA")));
    }

    #[test]
    fn it_matches_account_as_last_element() {
        let input = "Assets:Savings:ISA";
        assert_eq!(account(input), Ok(("", input)));
    }
}
