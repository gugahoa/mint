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
    pub fn new(host: String, client: U) -> Self {
        let robot = RobotFileParser::new(&host);
        Client {
            _marker: PhantomData,
            client,
            host,
            crawler_name: "".into(),
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
            crawler_name: "".into(),
            robot,
        }
    }
}

impl<'a, U: HTTPClient> Client<'a, RobotFetched, U> {
    pub async fn get(&self, path: String) -> Result<String, String> {
        Ok(format!("{}{}{}", self.host, self.crawler_name, path))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
