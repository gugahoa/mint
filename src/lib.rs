use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;
use std::time::Instant;

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
    robot: RefCell<RobotFileParser<'a>>,
    pub(crate) last_fetch: Instant,
    refetch_period: Duration,
}

impl<'a, U: HTTPClient> Client<'a, RobotUnfetched, U> {
    pub fn new(host: String, crawler_name: String, refetch_period: Duration, client: U) -> Self {
        let robot = RobotFileParser::new(&host);
        Client {
            _marker: PhantomData,
            client,
            host,
            crawler_name,
            robot: RefCell::new(robot),
            last_fetch: Instant::now(),
            refetch_period,
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
            robot: RefCell::new(robot),
            last_fetch: Instant::now(),
            refetch_period: self.refetch_period,
        }
    }
}

impl<'a, U: HTTPClient> Client<'a, RobotFetched, U> {
    pub async fn get(&self, path: &str) -> Result<String, String> {
        if self.robot.borrow().can_fetch(&self.crawler_name[..], path) {
            return Err("can't fetch".into());
        }

        if self.last_fetch.elapsed() > self.refetch_period {
            self.refetch_robots().await;
        }

        Ok("".into())
    }

    async fn refetch_robots(&self) {
        let response: String = self
            .client
            .get(&format!("{}/robots.txt", &self.host), &self.crawler_name)
            .await
            .into();

        let robot = RobotFileParser::new(&self.host);
        let lines: Vec<&str> = response.split("\n").collect();
        robot.parse(&lines);

        let mut self_robot = self.robot.borrow_mut();
        *self_robot = robot;
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
Host: https://eshop-prices.com
";

    use std::rc::Rc;

    use super::*;
    #[derive(Debug)]
    struct TestClient {
        number_of_fetches: RefCell<u32>,
    }

    impl HTTPClient for Rc<TestClient> {
        type Output = String;
        fn get(
            &self,
            _url: &str,
            _user_agent: &str,
        ) -> Pin<Box<dyn Future<Output = Self::Output>>> {
            let new_number_of_fetches = *self.number_of_fetches.borrow() + 1;
            let mut number_of_fetches = self.number_of_fetches.borrow_mut();
            *number_of_fetches = new_number_of_fetches;
            Box::pin(async { ROBOTS_TXT.into() })
        }
    }

    #[tokio::test]
    async fn cant_fetch() {
        let test_client = Rc::new(TestClient {
            number_of_fetches: RefCell::new(0),
        });

        let client = Client::new(
            "https://eshop-prices.com".into(),
            "dotbot".into(),
            Duration::new(60 * 60, 0),
            test_client.clone(),
        );
        let client = client.fetch_robots().await;

        assert_eq!(client.get("/").await, Err("can't fetch".into()));
        assert_eq!(*test_client.number_of_fetches.borrow(), 1);
    }

    #[tokio::test]
    async fn refetches_after_period() {
        let test_client = Rc::new(TestClient {
            number_of_fetches: RefCell::new(0),
        });

        let refetch_period = Duration::new(60 * 60, 0);
        let client = Client::new(
            "https://eshop-prices.com".into(),
            "allowed robot".into(),
            refetch_period,
            test_client.clone(),
        );
        let client = client.fetch_robots().await;
        let client = Client {
            last_fetch: Instant::now() - refetch_period,
            ..client
        };

        dbg!(&client);
        assert_eq!(client.get("/").await, Ok("".into()));
        assert_eq!(*test_client.number_of_fetches.borrow(), 2);
    }
}
