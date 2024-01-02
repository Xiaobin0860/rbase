use crate::{Ev, Value};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Kvpair {
    pub key: String,
    pub val: Value,
}

mod memory;
pub use memory::MemTable;
mod redisdb;
pub use redisdb::RedisDb;

/// 存储接口
pub trait Storage: Send + Sync + 'static {
    type Error: std::fmt::Debug;

    /// 从`Hashtable`中获取`key`的`Value`
    fn get(&self, table: &str, key: &str) -> Result<Value, Self::Error>;

    /// 向`Hashtable`中设置`key`的`Value`
    fn set(&self, table: &str, key: &str, value: Value) -> Result<(), Self::Error>;

    /// 从`Hashtable`中删除`key`
    fn del(&self, table: &str, key: &str) -> Result<(), Self::Error>;

    /// 获取`Hashtable`中所有的`Kvpair`
    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, Self::Error>;

    /// 遍历`Hashtable`返回`Kvpair`的Iterator
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, Self::Error>;
}

pub struct StorageIter<T> {
    inner: T,
}

impl<T> StorageIter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Iterator for StorageIter<T>
where
    T: Iterator,
    T::Item: Into<Kvpair>,
{
    type Item = Kvpair;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| item.into())
    }
}

impl Kvpair {
    pub fn new(key: impl Into<String>, val: impl Into<Value>) -> Self {
        Self {
            key: key.into(),
            val: val.into(),
        }
    }
}

impl From<(String, Value)> for Kvpair {
    fn from(data: (String, Value)) -> Self {
        Self::new(data.0, data.1)
    }
}

impl From<(String, Vec<u8>)> for Kvpair {
    fn from(data: (String, Vec<u8>)) -> Self {
        Self::new(data.0, data.1)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            ev: Some(Ev::String(value)),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self {
            ev: Some(Ev::String(value.into())),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self {
            ev: Some(Ev::Binary(value)),
        }
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Self {
            ev: Some(Ev::Binary(value.into())),
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self {
            ev: Some(Ev::Integer(value)),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            ev: Some(Ev::Float(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memtable_basic_interface_should_work() {
        let store = MemTable::new();
        test_basi_interface(store);
    }

    #[test]
    fn memtable_get_all_should_work() {
        let store = MemTable::new();
        test_get_all(store);
    }

    #[test]
    fn memtable_iter_should_work() {
        let store = MemTable::new();
        test_get_iter(store);
    }

    const ADDR: &str = "redis://test.at:7000/";

    #[test]
    fn redis_basic_interface_should_work() {
        let store = RedisDb::new(ADDR).unwrap();
        test_basi_interface(store);
    }

    #[test]
    fn redis_get_all_should_work() {
        let store = RedisDb::new(ADDR).unwrap();
        test_get_all(store);
    }

    fn test_basi_interface(store: impl Storage) {
        let v = store.set("t1", "hello", "world".into());
        assert!(v.is_ok());

        let v = store.get("t1", "hello");
        assert_eq!(v.unwrap(), "world".into());

        // get 不存在的 key 或者 table 会得到 None
        assert_eq!(None, store.get("t1", "hello1").unwrap().ev);
        assert!(store.get("t2", "hello1").unwrap().ev.is_none());

        // 删除存在，或不存在的table或key都不会报错
        assert!(store.del("t1", "hello").is_ok());
        assert!(store.del("t1", "hello1").is_ok());
        assert!(store.del("t2", "hello").is_ok());
    }

    fn test_get_all(store: impl Storage) {
        store.set("t2", "k1", "v1".into()).unwrap();
        store.set("t2", "k2", "v2".into()).unwrap();
        let mut data = store.get_all("t2").unwrap();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(data, vec![Kvpair::new("k1", "v1"), Kvpair::new("k2", "v2")])
    }

    fn test_get_iter(store: impl Storage) {
        store.set("t2", "k1", "v1".into()).unwrap();
        store.set("t2", "k2", "v2".into()).unwrap();
        let mut data: Vec<_> = store.get_iter("t2").unwrap().collect();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(data, vec![Kvpair::new("k1", "v1"), Kvpair::new("k2", "v2")])
    }
}
