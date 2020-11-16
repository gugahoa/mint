use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::bytes::complete::{is_not, tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::eof;
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::IResult;

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn is_user_agent(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '-' || c == '_' || c == '*'
}

pub(crate) fn user_agent(input: &str) -> IResult<&str, &str> {
    map(
        separated_pair(
            ws(tag_no_case("user-agent")),
            tag(":"),
            ws(take_while(is_user_agent)),
        ),
        |(_, user_agent)| user_agent,
    )(input)
}

fn allow_rule(input: &str) -> IResult<&str, &str> {
    map(alt((tag_no_case("allow"), tag_no_case("alow"))), |_| {
        "allow"
    })(input)
}

fn disallow_rule(input: &str) -> IResult<&str, &str> {
    map(
        alt((tag_no_case("disallow"), tag_no_case("disalow"))),
        |_| "disallow",
    )(input)
}

fn rule_name(input: &str) -> IResult<&str, &str> {
    let (input, rule_name) = ws(alt((allow_rule, disallow_rule, is_not(" :"))))(input)?;
    Ok((input, rule_name))
}

pub(crate) fn rule(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, rule_name) = rule_name(input)?;
    let (input, rule_value) =
        preceded(pair(tag(":"), opt(multispace0)), alt((eof, is_not(" #\n"))))(input)?;
    Ok((input, (rule_name, rule_value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_test() {
        assert_eq!(user_agent("user-agent:         mybot"), Ok(("", "mybot")));
        assert_eq!(
            user_agent("     user-agent       :         mybot"),
            Ok(("", "mybot"))
        );

        assert_eq!(user_agent("user-agent: my-bot"), Ok(("", "my-bot")));
    }

    #[test]
    fn allow_rule_test() {
        assert_eq!(allow_rule("alow"), Ok(("", "allow")));
        assert_eq!(allow_rule("allow"), Ok(("", "allow")));
    }

    #[test]
    fn disallow_rule_test() {
        assert_eq!(disallow_rule("disalow"), Ok(("", "disallow")));
        assert_eq!(disallow_rule("disallow"), Ok(("", "disallow")));
    }

    #[test]
    fn rule_name_test() {
        assert_eq!(rule_name("  alow  "), Ok(("", "allow")));
        assert_eq!(rule_name("  allow  "), Ok(("", "allow")));
        assert_eq!(rule_name("allow  "), Ok(("", "allow")));
        assert_eq!(rule_name("  allow"), Ok(("", "allow")));
        assert_eq!(rule_name("allow"), Ok(("", "allow")));

        assert_eq!(rule_name("  disalow  "), Ok(("", "disallow")));
        assert_eq!(rule_name("  disallow  "), Ok(("", "disallow")));
        assert_eq!(rule_name("disallow  "), Ok(("", "disallow")));
        assert_eq!(rule_name("  disallow"), Ok(("", "disallow")));
        assert_eq!(rule_name("disallow"), Ok(("", "disallow")));

        assert_eq!(rule_name("crawl-delay"), Ok(("", "crawl-delay")));
        assert_eq!(rule_name("  crawl-delay"), Ok(("", "crawl-delay")));
        assert_eq!(rule_name("crawl-delay  "), Ok(("", "crawl-delay")));
        assert_eq!(rule_name("  crawl-delay  "), Ok(("", "crawl-delay")));
    }

    #[test]
    fn rule_test() {
        assert_eq!(
            rule(" alow   :    /some-path"),
            Ok(("", ("allow", "/some-path")))
        );

        assert_eq!(
            rule(" alow   :    /some-path\n"),
            Ok(("\n", ("allow", "/some-path")))
        );

        assert_eq!(
            rule(" disalow   :    /some-path"),
            Ok(("", ("disallow", "/some-path")))
        );

        assert_eq!(
            rule(" disalow   :    /some-path\n"),
            Ok(("\n", ("disallow", "/some-path")))
        );

        assert_eq!(rule(" disalow   :"), Ok(("", ("disallow", ""))));
        assert_eq!(rule("crawl-delay: 1s"), Ok(("", ("crawl-delay", "1s"))));
    }
}
