pub mod configs {
    use async_std::fs;
    use async_std::task::block_on;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Config {
        pub controllers: HashMap<String, StripConfig>,
        pub macros: Vec<MacroConfig>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MacroConfig {
        pub name: String,
        pub actions: Vec<(String, String)>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct StripConfig {
        pub ip: String,
        pub port: u16,
        // pub friendly_name: String, friendly_name is now the key in the HashMap instead
        pub is_rgbw: bool,
    }

    impl Config {
        async fn load(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
            let file = fs::read_to_string(file).await?;
            let config = serde_yaml::from_str(&file)?;
            Ok(config)
        }
    }

    impl Default for Config {
        fn default() -> Self {
            block_on(Self::load("config.yaml")).unwrap()
        }
    }

    impl From<&str> for Config {
        fn from(file: &str) -> Self {
            block_on(Self::load(file)).unwrap()
        }
    }
}
