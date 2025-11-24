// // 调度任务
// // 开启一个定时任务，每1秒执行一次，从出站信息表中查询未发送和发送失败的消息，向redis pub/sub中发送
// use crate::{
//     models::outbox_message::{OutboxMessage, OutboxMessageSendFail, OutboxMessageSendSuccess},
//     services::outbox_message_service::OutboxMessageService,
// }
// pub(crate) async fn schedule_outbox_message_task(
//     outbox_message_service: Arc<OutboxMessageService>,
// ) {
//     loop {
//         // 获取未发送和发送失败的消息
//         let res = outbox_message_service
//             .list_by_status(vec![SendStatus::NotSent, SendStatus::SendFail])
//             .await;
//         if res.is_err() {
//             continue;
//         }
//     }
// }
