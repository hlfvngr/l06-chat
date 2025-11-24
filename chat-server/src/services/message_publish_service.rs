use chrono::Utc;
use std::sync::Arc;
use tracing::error;

use crate::{
    error::AppError,
    models::outbox_message::{OutboxMessage, OutboxMessageSendFail, OutboxMessageSendSuccess},
    mq::producers::outbox_message_producer::OutboxMessageProducer,
    services::outbox_message_service::OutboxMessageService,
};

#[derive(Debug)]
pub(crate) struct MessagePublishService {
    pub(crate) outbox_message_producer: Arc<OutboxMessageProducer>,
    pub(crate) outbox_message_service: Arc<OutboxMessageService>,
}

impl MessagePublishService {
    pub fn new(
        outbox_message_producer: Arc<OutboxMessageProducer>,
        outbox_message_service: Arc<OutboxMessageService>,
    ) -> Self {
        Self {
            outbox_message_producer,
            outbox_message_service,
        }
    }
}
impl MessagePublishService {
    pub async fn send_message(&self, msg: &OutboxMessage) -> Result<(), AppError> {
        match self.outbox_message_producer.send(msg.clone()).await {
            Ok(_) => {
                self.outbox_message_service
                    .update_success_messages(vec![OutboxMessageSendSuccess {
                        id: msg.id,
                        send_success_time: Utc::now(),
                    }])
                    .await?;
            }
            Err(e) => {
                error!("send message error: {}", e);
                let last_retry_time = Utc::now();
                let next_retry_time = last_retry_time + chrono::Duration::minutes(5);
                let send_fail_reason = e.to_string();
                self.outbox_message_service
                    .update_failed_messages(vec![OutboxMessageSendFail {
                        id: msg.id,
                        last_retry_time,
                        send_fail_reason,
                        next_retry_time,
                    }])
                    .await?;
            }
        };
        Ok(())
    }
}
