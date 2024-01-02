use crate::{BaseError, Kvpair, Storage, StorageIter, Value};
use dashmap::{mapref::one::Ref, DashMap};

/// 使用 DashMap 构建的 MemTable，实现了 Storage trait
#[derive(Clone, Debug, Default)]
pub struct MemTable {
    tables: DashMap<String, DashMap<String, Value>>,
}

impl MemTable {
    /// 创建一个缺省的 MemTable
    pub fn new() -> Self {
        Self::default()
    }

    /// 如果名为 name 的 hash table 不存在，则创建，否则返回
    fn get_or_create_table(&self, name: &str) -> Ref<String, DashMap<String, Value>> {
        match self.tables.get(name) {
            Some(table) => table,
            None => {
                let entry = self.tables.entry(name.into()).or_default();
                entry.downgrade()
            }
        }
    }
}

impl Storage for MemTable {
    type Error = BaseError;

    fn get(&self, table: &str, key: &str) -> Result<Value, Self::Error> {
        let table = self.get_or_create_table(table);
        Ok(table
            .get(key)
            .map(|v| v.value().clone())
            .unwrap_or_default())
    }

    fn set(&self, table: &str, key: &str, value: Value) -> Result<(), Self::Error> {
        let table = self.get_or_create_table(table);
        table.insert(key.into(), value);
        Ok(())
    }

    fn del(&self, table: &str, key: &str) -> Result<(), Self::Error> {
        let table = self.get_or_create_table(table);
        table.remove(key);
        Ok(())
    }

    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, Self::Error> {
        let table = self.get_or_create_table(table);
        Ok(table
            .iter()
            .map(|v| Kvpair::new(v.key(), v.value().clone()))
            .collect())
    }

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, Self::Error> {
        // 使用 clone() 来获取 table 的 snapshot
        let table = self.get_or_create_table(table).clone();
        let iter = StorageIter::new(table.into_iter());
        Ok(Box::new(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_create_table_should_work() {
        let store = MemTable::new();
        assert!(!store.tables.contains_key("t1"));
        store.get_or_create_table("t1");
        assert!(store.tables.contains_key("t1"));
    }
}
