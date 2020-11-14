use std::marker::PhantomData;

struct RobotFetched;
struct RobotUnfetched;

pub struct Client<T> {
    _marker: PhantomData<T>,
    client: String,
    host: String,
}

impl Client<RobotUnfetched> {
    pub fn new(host: String) -> Self {
        Client {
            client: "".into(),
            host,
            _marker: PhantomData,
        }
    }

    pub async fn fetch_robots(self) -> Client<RobotFetched> {
        Client {
            client: self.client,
            host: self.host,
            _marker: PhantomData,
        }
    }
}

impl Client<RobotFetched> {
    pub async fn get(&self, path: String) -> Result<String, String> {
        dbg!(&self.client);
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
