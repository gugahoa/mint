use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use robotparser::RobotFileParser;

#[derive(Debug)]
struct RobotFetched;
#[derive(Debug)]
struct RobotUnfetched;

// TODO: Delete me
#[derive(Debug)]
struct Empty;

pub trait HTTPClient {
    type Output: Into<String>;
    fn get(&self, url: &str, user_agent: &str) -> Pin<Box<dyn Future<Output = Self::Output>>>;
}

impl HTTPClient for Empty {
    type Output = String;
    fn get(&self, url: &str, user_agent: &str) -> Pin<Box<dyn Future<Output = Self::Output>>> {
        let ret = format!("{}{}", url, user_agent);
        Box::pin(async { ret })
    }
}

#[derive(Debug)]
pub struct Client<'a, T, U: HTTPClient> {
    _marker: PhantomData<T>,
    client: U,
    host: String,
    crawler_name: String,
    robot: RobotFileParser<'a>,
}

impl<'a, U: HTTPClient> Client<'a, RobotUnfetched, U> {
    pub fn new(host: String, crawler_name: String, client: U) -> Self {
        let robot = RobotFileParser::new(&host);
        Client {
            _marker: PhantomData,
            client,
            host,
            crawler_name,
            robot,
        }
    }

    pub async fn fetch_robots(self) -> Client<'a, RobotFetched, U> {
        let response: String = self
            .client
            .get(&format!("{}/robots.txt", &self.host), &self.crawler_name)
            .await
            .into();

        let robot = RobotFileParser::new(&self.host);
        let lines: Vec<&str> = response.split("\n").collect();
        robot.parse(&lines);

        Client {
            _marker: PhantomData,
            client: self.client,
            host: self.host,
            crawler_name: self.crawler_name,
            robot,
        }
    }
}

impl<'a, U: HTTPClient> Client<'a, RobotFetched, U> {
    pub async fn get(&self, path: &str) -> Result<String, String> {
        if !self.robot.can_fetch::<&str>(&self.crawler_name, path) {
            return Err("can't fetch".into());
        }
        Ok(format!("{}{}{}", self.host, self.crawler_name, path))
    }
}

#[cfg(test)]
mod tests {
    const ROBOTS_TXT: &'static str = "
        User-agent: SemrushBot
        User-agent: SemrushBot-SA
        User-agent: dotbot
        User-agent: rogerbot
        User-agent: AhrefsBot
        User-agent: MJ12bot
        User-agent: SMTBot
        User-agent: BLEXBot
        Disallow: /
        Crawl-Delay: 1

        User-agent: *
        Host: https://eshop-prices.com";

    use super::*;
    #[derive(Debug)]
    struct TestClient;

    impl HTTPClient for TestClient {
        type Output = String;
        fn get(
            &self,
            _url: &str,
            _user_agent: &str,
        ) -> Pin<Box<dyn Future<Output = Self::Output>>> {
            Box::pin(async { ROBOTS_TXT.into() })
        }
    }

    #[tokio::test]
    async fn cant_fetch() {
        let client = Client::new(
            "https://eshop-prices.com".into(),
            "dotbot".into(),
            TestClient,
        );
        let client = client.fetch_robots().await;

        dbg!(&client);
        assert_eq!(client.get("/").await, Err("can't fetch".into()));
    }
}
