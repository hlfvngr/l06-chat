// src/redis_lock.rs
use redis::{Connection, RedisError, RedisResult, Script};
use std::time::Duration;

pub struct RedisLock<'a> {
    con: &'a mut Connection, // 使用 Option 以便在 Drop 中 take()
    key: String,
}

impl<'a> RedisLock<'a> {
    pub fn new<'b: 'a>(con: &'b mut Connection, key: impl Into<String>) -> Self {
        RedisLock {
            con,
            key: key.into(),
        }
    }
    /// 尝试获取锁
    pub fn acquire(&mut self, ttl: Option<Duration>) -> Result<(), anyhow::Error> {
        let ttl_millis = if let Some(ttl) = ttl {
            ttl.as_millis() as i64
        } else {
            -1
        };

        // 原子加锁：SET key value NX PX ttl
        let result: Result<Option<String>, RedisError> = redis::cmd("SET")
            .arg(&self.key)
            .arg("1")
            .arg("NX")
            .arg("PX")
            .arg(ttl_millis)
            .query(self.con);

        // info!("Lock result: {:?}", result);

        match result {
            Ok(Some(_)) => {
                // 成功获取锁
                Ok(())
            }
            Ok(None) => {
                // 锁已存在
                Err(anyhow::anyhow!("Lock already exists"))
            }
            Err(e) => {
                // Redis 错误
                Err(anyhow::anyhow!("Redis error: {}", e))
            }
        }
    }

    /// 手动释放锁（可选）
    pub fn release(&mut self) -> Result<bool, anyhow::Error> {
        let result = Self::do_release(self.con, &self.key, "1");
        match result {
            Ok(b) => Ok(b),
            Err(e) => Err(anyhow::anyhow!("Redis error: {}", e)),
        }
    }

    /// 实际执行释放逻辑（供 Drop 和手动 release 使用）
    fn do_release(con: &mut Connection, key: &str, value: &str) -> RedisResult<bool> {
        const RELEASE_SCRIPT: &str = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let script = Script::new(RELEASE_SCRIPT);
        let result: i32 = script.key(key).arg(value).invoke(con)?;
        Ok(result == 1)
    }
}

impl<'a> Drop for RedisLock<'a> {
    fn drop(&mut self) {
        // 在 Drop 中同步释放锁
        let _ = Self::do_release(self.con, &self.key, "1");
        // 注意：这里忽略错误（如网络断开），因为锁有 TTL 会自动过期
    }
}
