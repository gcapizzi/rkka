use anyhow::Result;

use crate::redis;

pub struct ConcurrentHashMap {
    map: scc::HashMap<String, String>,
}

impl ConcurrentHashMap {
    pub fn new() -> Self {
        ConcurrentHashMap {
            map: scc::HashMap::new(),
        }
    }
}

impl redis::Engine for ConcurrentHashMap {
    fn call(&self, command: redis::Command) -> redis::Result {
        match command {
            redis::Command::Get { key: redis::Key(k) } => self
                .get(k)
                .map(|v| redis::Result::BulkString(v))
                .unwrap_or(redis::Result::Null),
            redis::Command::Set {
                key: redis::Key(k),
                value: redis::String(v),
            } => {
                self.set(k, v);
                redis::Result::Ok
            }
            redis::Command::Client => redis::Result::Ok,
            redis::Command::Incr { key: redis::Key(k) } => self
                .incr(k)
                .map(|v| redis::Result::Integer(v))
                .unwrap_or_else(|e| redis::Result::Error(e.to_string())),
        }
    }
}

impl ConcurrentHashMap {
    fn get(&self, key: String) -> Option<String> {
        self.map.read(&key, |_, v| v.clone())
    }

    fn set(&self, key: String, value: String) {
        self.map.upsert(key, value);
    }

    fn incr(&self, key: String) -> Result<i64> {
        if let Some(mut value) = self.map.get(&key) {
            let mut new_value: i64 = value.parse()?;
            new_value += 1;
            *value = new_value.to_string();
            Ok(new_value)
        } else {
            self.map.upsert(key, "1".to_string());
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::Engine;
    use crate::redis::Key;
    use crate::redis::String;

    #[test]
    fn test_set_and_get() {
        let redis = ConcurrentHashMap::new();

        let result = redis.call(redis::Command::Set {
            key: Key("key".to_string()),
            value: String("value".to_string()),
        });
        assert_eq!(result, redis::Result::Ok);

        let result = redis.call(redis::Command::Get {
            key: Key("key".to_string()),
        });
        assert_eq!(result, redis::Result::BulkString("value".to_string()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let redis = ConcurrentHashMap::new();

        let result = redis.call(redis::Command::Get {
            key: Key("nonexistent".to_string()),
        });
        assert_eq!(result, redis::Result::Null);
    }

    #[test]
    fn test_client() {
        let redis = ConcurrentHashMap::new();

        let result = redis.call(redis::Command::Client);
        assert_eq!(result, redis::Result::Ok);
    }

    #[test]
    fn test_incr() {
        let redis = ConcurrentHashMap::new();

        let result = redis.call(redis::Command::Incr {
            key: Key("counter".to_string()),
        });
        assert_eq!(result, redis::Result::Integer(1));

        let result = redis.call(redis::Command::Incr {
            key: Key("counter".to_string()),
        });
        assert_eq!(result, redis::Result::Integer(2));

        let result = redis.call(redis::Command::Get {
            key: Key("counter".to_string()),
        });
        assert_eq!(result, redis::Result::BulkString("2".to_string()));
    }
}
