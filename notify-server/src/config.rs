use std::{fs::File, ops::Deref};

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: Config,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
}

impl Deref for AppConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        let env_config = std::env::var("NOTIFY_CONFIG_FILE").unwrap_or("".to_owned());
        let f = match (
            File::open("./notify.yaml"),
            File::open("/etc/chat/notify.yaml"),
            File::open(env_config),
        ) {
            (Ok(f), _, _) => f,
            (_, Ok(f), _) => f,
            (_, _, Ok(f)) => f,
            (_, _, _) => bail!("No config file found"),
        };

        let config: AppConfig = serde_yaml_bw::from_reader(f)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = AppConfig::load().unwrap();
        assert_eq!(config.port, 8889);
    }
}
