use std::sync::Arc;

use crate::AppState;
use anyhow::Result;
use chat_core::event::ChatEvent;
use tokio_stream::StreamExt;
use tracing::info;

pub async fn start_background_task(state: AppState) -> Result<()> {
    tokio::spawn(async move {
        let client =
            redis::Client::open(&*state.app_config.redis.url).expect(" can't connect redis");
        let mut pubsub_conn = client
            .get_async_pubsub()
            .await
            .expect("can't get redis pubsub connection");

        let _: () = pubsub_conn
            .subscribe("chat")
            .await
            .expect("can't subscribe redis pubsub");
        info!("start to subscribe redis pubsub");
        let mut pubsub_stream = pubsub_conn.on_message();

        while let Some(msg) = pubsub_stream.next().await {
            info!(
                "receive message: {}, payload: {}",
                msg.get_channel_name(),
                msg.get_payload::<String>().unwrap()
            );

            let payload = msg.get_payload::<String>().unwrap();
            let event: ChatEvent = serde_json::from_str(&payload).unwrap();
            process_event(event, &state).await;
        }
    });

    Ok(())
}

async fn process_event(event: ChatEvent, state: &AppState) {
    let users = state.users.clone();
    let event = Arc::new(event);
    match event.as_ref() {
        ChatEvent::ChatCreate(msg) => {
            for user_id in msg.members.iter() {
                if let Some(user) = users.get(user_id)
                    && let Err(e) = user.value().send(Arc::clone(&event))
                {
                    info!("send message to user failed: {}", e);
                }
            }
        }
        ChatEvent::ChatDrop(msg) => {
            for user_id in msg.members.iter() {
                if let Some(user) = users.get(user_id)
                    && let Err(e) = user.value().send(Arc::clone(&event))
                {
                    info!("send message to user failed: {}", e);
                }
            }
        }
        ChatEvent::MessageSend(msg) => {
            for user_id in msg.members.iter() {
                if let Some(user) = users.get(user_id)
                    && let Err(e) = user.value().send(Arc::clone(&event))
                {
                    info!("send message to user failed: {}", e);
                }
            }
        }
        ChatEvent::UserJoin(msg) => {
            for user_id in msg.members.iter() {
                if let Some(user) = users.get(user_id)
                    && let Err(e) = user.value().send(Arc::clone(&event))
                {
                    info!("send message to user failed: {}", e);
                }
            }
        }
        ChatEvent::UserLeave(msg) => {
            for user_id in msg.members.iter() {
                if let Some(user) = users.get(user_id)
                    && let Err(e) = user.value().send(Arc::clone(&event))
                {
                    info!("send message to user failed: {}", e);
                }
            }
        }
    }
}
