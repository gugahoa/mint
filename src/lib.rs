use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;
use std::time::Instant;

use robotparser::RobotFileParser;

mod robots;

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
