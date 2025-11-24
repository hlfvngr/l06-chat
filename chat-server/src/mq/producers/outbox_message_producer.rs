use anyhow::Result;
use redis::AsyncCommands;

use crate::models::outbox_message::OutboxMessage;
#[derive(Debug)]
pub(crate) struct OutboxMessageProducer {
    pub(crate) redis_client: redis::Client,
}

impl OutboxMessageProducer {
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    pub async fn send(&self, message: OutboxMessage) -> Result<()> {
        let mut publish_conn = self.redis_client.get_multiplexed_async_connection().await?;
        let _: () = publish_conn.publish("chat", message.content).await?;
        Ok(())
    }
}
