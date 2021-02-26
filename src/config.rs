use std::env;

// retrieve a application configuration from the environment
pub fn get(key: &'static str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Fatal: the enviroment variable {} is required.", key))
}
