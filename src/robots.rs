mod parser;

#[derive(Debug, Eq, PartialEq)]
struct Group<'a> {
    user_agents: Vec<&'a str>,
    rules: Vec<(&'a str, &'a str)>,
}

fn parse(robots_txt: &str) -> Vec<Group> {
    let mut groups = vec![];

    let mut user_agents = vec![];
    let mut rules = vec![];

    let mut group_started = false;
    for line in robots_txt.lines() {
        if let Ok((_, user_agent)) = parser::user_agent(line) {
            group_started = true;
            user_agents.push(user_agent);
            continue;
        }

        if group_started {
            if line.trim() == "" {
                group_started = false;

                groups.push(Group {
                    rules: std::mem::take(&mut rules),
                    user_agents: std::mem::take(&mut user_agents),
                });
                continue;
            }

            if let Ok((_, rule)) = parser::rule(line) {
                rules.push(rule);
                continue;
            }
        }
    }

    if group_started {
        groups.push(Group { rules, user_agents });
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cratesio_test() {
        let robots_txt = include_str!("../tests/robotstxt/cratesio_robots.txt");
        assert_eq!(
            parse(robots_txt),
            vec![Group {
                rules: vec![("disallow", "")],
                user_agents: vec!["*"],
            }]
        );
    }

    #[test]
    fn parse_eshop_prices_test() {
        let robots_txt = include_str!("../tests/robotstxt/eshop_prices_robots.txt");
        assert_eq!(
            parse(robots_txt),
            vec![
                Group {
                    user_agents: vec![
                        "SemrushBot",
                        "SemrushBot-SA",
                        "dotbot",
                        "rogerbot",
                        "AhrefsBot",
                        "MJ",
                        "SMTBot",
                        "BLEXBot"
                    ],
                    rules: vec![("disallow", "/"), ("Crawl-Delay", "1")]
                },
                Group {
                    user_agents: vec!["*"],
                    rules: vec![("Host", "https://eshop-prices.com")]
                }
            ]
        );
    }
}
