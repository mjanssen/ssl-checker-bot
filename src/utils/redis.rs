use redis::{self, Client, Connection};
use redis::{Commands, RedisError};

pub enum Database {
    User = 0,
    Domains = 1,
}

#[derive(Clone)]
pub struct RedisClient {
    client: Client,
}
impl RedisClient {
    pub fn new() -> Self {
        match redis::Client::open("redis://127.0.0.1/") {
            Ok(client) => Self { client },
            Err(error) => panic!("Could not create redis client: {}", error),
        }
    }

    pub fn get_connection(&self) -> Connection {
        match self.client.get_connection() {
            Ok(con) => con,
            Err(error) => panic!("Could not establish connection with Redis: {}", error),
        }
    }

    pub fn set_database(&self, con: &mut Connection, db: Database) {
        // Make the current connection connect to the messages database
        let _: Result<(), redis::RedisError> = redis::cmd("SELECT").arg(db as u8).query(con);
    }
}

type RedisResult = redis::RedisResult<()>;

pub struct RedisActions {}
impl RedisActions {
    pub fn store_user(client: &RedisClient, user_id: &str) -> RedisResult {
        let mut con = client.get_connection();
        client.set_database(&mut con, Database::User);

        con.set(user_id, 1)?;
        Ok(())
    }

    pub fn store_domain(client: &RedisClient, user_id: &str, domain: &str) -> RedisResult {
        let mut con = client.get_connection();
        client.set_database(&mut con, Database::Domains);

        let existing_value: Option<String> = con.get(user_id)?;
        let new_value = match existing_value {
            Some(value) => format!("{value},{domain}"),
            None => domain.to_string(),
        };

        con.set(user_id, new_value)?;
        Ok(())
    }

    pub fn remove_domain(
        client: &RedisClient,
        user_id: &str,
        domain: &str,
    ) -> Result<Option<()>, RedisError> {
        let mut con = client.get_connection();
        client.set_database(&mut con, Database::Domains);

        let existing_value: Option<String> = con.get(user_id)?;
        let new_value = match existing_value {
            // Make sure to fix the string
            Some(value) => {
                if value.contains(domain) == false {
                    return Ok(None);
                }

                value
                    .replace(domain, "")
                    .replace(",,", ",")
                    .trim_matches(',')
                    .to_string()
            }
            // Early return if there are no values for this user
            None => return Ok(None),
        };
        con.set(user_id, new_value)?;

        Ok(Some(()))
    }

    pub fn get_domains(client: &RedisClient, user_id: &str) -> Result<Option<String>, RedisError> {
        let mut con = client.get_connection();
        client.set_database(&mut con, Database::Domains);

        Ok(con.get(user_id)?)
    }
}
