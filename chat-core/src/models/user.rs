use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurUser {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
}
