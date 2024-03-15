use std::env;

pub fn get_environment_variable(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_e) => {
            panic!("Missing environment variable: {}", key);
        }
    }
}
