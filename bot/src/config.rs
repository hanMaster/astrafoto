use crate::error::Error;
use crate::error::Result;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|err| {
            panic!("FATAL - WHILE LOADING Config -cause: {:?}", err);
        })
    })
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Eq, PartialEq)]
pub struct Config {
    pub API_URL: String,
    pub ID_INSTANCE: String,
    pub API_TOKEN_INSTANCE: String,
    pub ADMIN_CHAT_ID: String,
    pub WORKER_PORT: u16,
    pub WORKER_URL: String,
    pub SHOP_ADDRESS: String,
    pub SHOP_PHONE: String,
    pub NO_FILES_TIMEOUT: u64,
    pub REPEAT_COUNT: i32,
    pub REPEAT_TIMEOUT: u64,
    pub HOOK_PORT: u16,
    pub PAPER_PROMPT: String,
    pub SECRET_TOKEN: String,
    pub SIZE_PROMPT: String,
    pub FILES_DONE_PROMPT: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        // dotenv().expect("dotenv init failed");
        let worker_port = get_env_as_parse("WORKER_PORT")?;
        Ok(Config {
            API_URL: get_env("API_URL")?,
            ID_INSTANCE: get_env("ID_INSTANCE")?,
            API_TOKEN_INSTANCE: get_env("API_TOKEN_INSTANCE")?,
            ADMIN_CHAT_ID: get_env("ADMIN_CHAT_ID")?,
            WORKER_PORT: worker_port,
            WORKER_URL: format!("http://worker:{}/", worker_port),
            SHOP_ADDRESS: get_env("SHOP_ADDRESS")?,
            SHOP_PHONE: get_env("SHOP_PHONE")?,
            NO_FILES_TIMEOUT: get_env_as_parse("NO_FILES_TIMEOUT")?,
            REPEAT_COUNT: get_env_as_parse("REPEAT_COUNT")?,
            REPEAT_TIMEOUT: get_env_as_parse("REPEAT_TIMEOUT")?,
            HOOK_PORT: get_env_as_parse("HOOK_PORT")?,
            SECRET_TOKEN: get_env("SECRET_TOKEN")?,
            PAPER_PROMPT: get_env("PAPER_PROMPT")?,
            SIZE_PROMPT: get_env("SIZE_PROMPT")?,
            FILES_DONE_PROMPT: get_env("FILES_DONE_PROMPT")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_as_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}