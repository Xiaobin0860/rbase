use crate::{BaseError, Ev, Kvpair, Storage};
use prost::Message;
use redis::{Client as RedisClient, Commands};
use std::{collections::HashMap, ops::Deref};

/// Redis存储, 使用Hash类型实现
#[derive(Clone, Debug)]
pub struct RedisDb(RedisClient);

impl RedisDb {
    pub fn new(addr: &str) -> Result<Self, BaseError> {
        let client = RedisClient::open(addr)?;
        Ok(Self(client))
    }
}

impl Deref for RedisDb {
    type Target = RedisClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl redis::ToRedisArgs for crate::Value {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self.ev.as_ref() {
            Some(Ev::String(s)) => out.write_arg(s.as_bytes()),
            Some(Ev::Binary(b)) => out.write_arg(b.as_slice()),
            Some(Ev::Integer(i)) => out.write_arg_fmt(i),
            Some(Ev::Float(f)) => out.write_arg_fmt(f),
            Some(Ev::Bool(b)) => out.write_arg(if *b { b"1" } else { b"0" }),
            None => out.write_arg(b""),
        }
    }
}

impl Storage for RedisDb {
    type Error = BaseError;

    fn get(&self, table: &str, key: &str) -> Result<crate::Value, Self::Error> {
        let mut conn = self.get_connection()?;
        let v: Vec<u8> = conn.hget(table, key)?;

        Ok(Message::decode(&v[..])?)
    }

    fn set(&self, table: &str, key: &str, value: crate::Value) -> Result<(), Self::Error> {
        let mut conn = self.get_connection()?;
        conn.hset(table, key, value.encode_to_vec())?;
        Ok(())
    }

    fn del(&self, table: &str, key: &str) -> Result<(), Self::Error> {
        let mut conn = self.get_connection()?;
        conn.hdel(table, key)?;
        Ok(())
    }

    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, Self::Error> {
        let mut conn = self.get_connection()?;
        let map: HashMap<String, Vec<u8>> = conn.hgetall(table)?;
        let pairs = map
            .into_iter()
            .map(|(k, v)| {
                let v: crate::Value = Message::decode(&v[..]).unwrap_or_default();
                Kvpair::new(k, v)
            })
            .collect();
        Ok(pairs)
    }

    fn get_iter(&self, _table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, Self::Error> {
        unimplemented!("redis storage get_iter")
    }
}
