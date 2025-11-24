use std::fs::File;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub file_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub secret_key: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        let env_config = std::env::var("CHAT_CONFIG_FILE").unwrap_or("".to_owned());
        let f = match (
            File::open("./chat.yaml"),
            File::open("/etc/chat/chat.yaml"),
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
    use std::time::Duration;

    use chat_core::{
        models::user::CurUser,
        utils::jwt::{DecodingKey, EncodingKey},
    };

    use redis::AsyncCommands;
    use tokio::time::sleep;
    use tokio_stream::StreamExt;

    use super::*;

    #[test]
    fn test_load_config() {
        let config = AppConfig::load().unwrap();
        assert_eq!(config.server.port, 8888);
        //assert!(config
        //    .db_url
        //    .starts_with("mysql://root:lotto1988@192.168.2.118:3306"));
        assert_eq!(config.server.file_base_url, "http://localhost:8080/files/");
    }

    #[test]
    fn test_auth_encode_decode() {
        let config = AppConfig::load().unwrap();
        let encoding_key = EncodingKey::load(&config.auth.secret_key).unwrap();

        let user = CurUser {
            id: 1,
            ws_id: 1,
            fullname: "test".to_owned(),
            email: "".to_owned(),
        };

        let token = encoding_key.sign(user).unwrap();
        let decoding_key = DecodingKey::load(&config.auth.public_key).unwrap();

        let u = decoding_key.verify(&token).unwrap();
        assert_eq!(u.id, 1);
    }

    #[test]
    fn test_redis_connect() {
        let config = AppConfig::load().unwrap();
        println!("{:?}", config.redis);
        let client = redis::Client::open(config.redis.url).unwrap();
        let mut con = client.get_connection().unwrap();
        redis::cmd("SET")
            .arg("my_key")
            .arg("my_value")
            .exec(&mut con)
            .unwrap();
        let value: String = redis::cmd("GET").arg("my_key").query(&mut con).unwrap();
        assert_eq!(value, "my_value");
    }

    #[tokio::test]
    async fn test_redis_pubsub() {
        let config = AppConfig::load().unwrap();
        let client = redis::Client::open(config.redis.url).unwrap();

        let mut publish_conn = client.get_multiplexed_async_connection().await.unwrap();
        let mut pubsub_conn = client.get_async_pubsub().await.unwrap();

        let _: () = pubsub_conn.subscribe("wavephone").await.unwrap();
        let mut pubsub_stream = pubsub_conn.on_message();
        for _ in 0..100 {
            let _: () = publish_conn.publish("wavephone", "banana").await.unwrap();
            sleep(Duration::from_secs(1)).await;
        }
        let pubsub_msg: String = pubsub_stream.next().await.unwrap().get_payload().unwrap();
        assert_eq!(&pubsub_msg, "banana");
    }
}
