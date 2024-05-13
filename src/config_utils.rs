#[deprecated(
    since = "0.1.1",
    note = "This functionality has been moved to database backends, use those instead."
)]
pub mod configs {
    use async_std::fs;
    use async_std::task::block_on;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use tokio::sync::Mutex;

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

    #[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
    pub struct StripConfig {
        pub ip: String,
        pub port: u16,
        pub friendly_name: String,
        pub is_rgbw: bool,
    }

    pub trait Read {
        async fn read(&self) -> Config;
        async fn read_macros(&self) -> Vec<MacroConfig>;
        async fn read_controllers(&self) -> HashMap<String, StripConfig>;
    }

    impl Read for Mutex<Config> {
        async fn read(&self) -> Config {
            self.lock().await.clone()
        }

        async fn read_macros(&self) -> Vec<MacroConfig> {
            self.lock().await.macros.clone()
        }

        async fn read_controllers(&self) -> HashMap<String, StripConfig> {
            self.lock().await.controllers.clone()
        }
    }

    impl Config {
        async fn load(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
            let file = fs::read_to_string(file).await?;
            let config = serde_yaml::from_str(&file)?;
            Ok(config)
        }
    }

    impl Config {
        pub fn add_controller(
            &mut self,
            name: &String,
            friendly_name: &String,
            ip: &String,
            port: &u16,
            is_rgbw: &bool,
        ) {
            self.controllers.insert(
                name.clone(),
                StripConfig {
                    friendly_name: friendly_name.clone(),
                    ip: ip.clone(),
                    port: port.clone(),
                    is_rgbw: is_rgbw.clone(),
                },
            );
        }

        pub fn remove_controller(&mut self, name: String) {
            self.controllers.remove(&name);
        }

        pub async fn write(&self, file: &str) -> Result<(), std::io::Error> {
            let config = serde_yaml::to_string(&self).unwrap();
            fs::write(file, config).await
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
