use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

struct RobotFetched;
struct RobotUnfetched;

// TODO: Delete me
#[derive(Debug)]
struct Empty;

trait HTTPClient {
    type Output: Into<String>;
    fn get(url: String, user_agent: String) -> Pin<Box<dyn Future<Output = Self::Output>>>;
}

impl HTTPClient for Empty {
    type Output = String;
    fn get(url: String, _user_agent: String) -> Pin<Box<dyn Future<Output = Self::Output>>> {
        Box::pin(async { url })
    }
}

pub struct Client<T, U: HTTPClient> {
    _marker: PhantomData<T>,
    client: U,
    host: String,
}

impl<U: HTTPClient> Client<RobotUnfetched, U> {
    pub fn new(host: String, client: U) -> Self {
        Client {
            _marker: PhantomData,
            client,
            host,
        }
    }

    pub async fn fetch_robots(self) -> Client<RobotFetched, U> {
        Client {
            client: self.client,
            host: self.host,
            _marker: PhantomData,
        }
    }
}

impl<U: HTTPClient> Client<RobotFetched, U> {
    pub async fn get(&self, path: String) -> Result<String, String> {
        Ok(self.host.clone() + &path)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
