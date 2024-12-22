use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct General {
    pub block_time_secs: usize,
    pub keys_path: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub general: General
}
