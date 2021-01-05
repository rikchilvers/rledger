use nom::{branch::alt, bytes::complete::tag, sequence::terminated, IResult};

pub fn account(i: &str) -> IResult<&str, &str> {
    // terminated(
    //     end_of_account
    // )(i)
    Ok(("", ""))
}

fn end_of_account(i: &str) -> IResult<&str, &str> {
    alt((tag("  "), tag("\t"), tag(";"), tag("#")))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_matches_account() {
        let input = "Assets:Saving:ISA";
        assert_eq!(account(input), Ok(("", vec!["Assets", "Saving", "ISA"])));
    }
}
