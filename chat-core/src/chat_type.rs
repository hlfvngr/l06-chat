use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, encode::IsNull, mysql::MySqlTypeInfo, prelude::Type};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    Public,
    Private,
}

// 实现 Type：声明这个类型在 MySQL 中对应 VARCHAR/TEXT
impl Type<MySql> for ChatType {
    fn type_info() -> MySqlTypeInfo {
        // 对应 MySQL 的 TEXT 类型（也兼容 VARCHAR）
        <&str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        // 允许与字符串类型兼容
        <&str as Type<MySql>>::compatible(ty)
    }
}

// 实现 Encode：如何把 ChatType 写入数据库（转为字符串）
impl Encode<'_, MySql> for ChatType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> std::result::Result<IsNull, sqlx::error::BoxDynError> {
        let s: &str = self.into();
        <&str as Encode<MySql>>::encode(s, buf)
    }
}

// 实现 Decode：如何从数据库读取（从字符串解析）
impl<'r> Decode<'r, MySql> for ChatType {
    fn decode(
        value: <MySql as sqlx::Database>::ValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s: &str = <&str as Decode<MySql>>::decode(value)?;
        match s {
            "single" => std::result::Result::Ok(ChatType::Single),
            "group" => std::result::Result::Ok(ChatType::Group),
            "public" => std::result::Result::Ok(ChatType::Public),
            "private" => std::result::Result::Ok(ChatType::Private),
            _ => Err(format!("unknown ChatType variant: '{}'", s).into()),
        }
    }
}

// 辅助：ChatType → &str
impl From<&ChatType> for &'static str {
    fn from(t: &ChatType) -> Self {
        match t {
            ChatType::Single => "single",
            ChatType::Group => "group",
            ChatType::Public => "public",
            ChatType::Private => "private",
        }
    }
}
