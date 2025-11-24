use std::{convert::Infallible, time::Duration};

use axum::{
    Extension,
    extract::State,
    response::{Sse, sse::Event},
};
use chat_core::{event::ChatEvent, models::user::CurUser};
use futures::Stream;
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use tracing::{debug, info};

use crate::AppState;

pub(crate) async fn sse_handler(
    Extension(user): Extension<CurUser>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let users = state.users.clone();

    let rx = if let Some(tx) = users.get(&user.id) {
        tx.subscribe()
    } else {
        let (tx, rx) = tokio::sync::broadcast::channel(256);
        users.insert(user.id, tx);
        rx
    };

    info!("User {} subscribed", user.id);

    let stream = BroadcastStream::new(rx).filter_map(|v| v.ok()).map(|v| {
        let name = match v.as_ref() {
            ChatEvent::ChatCreate(_) => "ChatCreate",
            ChatEvent::ChatDrop(_) => "ChatDrop",
            ChatEvent::UserJoin(_) => "UserJoin",
            ChatEvent::UserLeave(_) => "UserLeave",
            ChatEvent::MessageSend(_) => "MessageSend",
        };
        let v = serde_json::to_string(&v).expect("Failed to serialize event");
        debug!("Sending event {}: {:?}", name, v);
        Ok(Event::default().data(v).event(name))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
