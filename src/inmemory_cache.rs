use actix::{prelude::Handler, Actor, Context, Message};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct ShardedCache<T> {
    shards: Vec<RwLock<HashMap<String, T>>>,
}

impl<T: Clone> ShardedCache<T> {
    pub fn new(shard_count: usize) -> Self {
        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(RwLock::new(HashMap::new()));
        }
        ShardedCache { shards }
    }

    fn hash(key: &str) -> usize {
        let mut hash = 0usize;
        for b in key.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as usize);
        }
        hash
    }

    fn get_shard(&self, key: &str) -> usize {
        Self::hash(key) % self.shards.len()
    }

    pub fn set(&self, key: &String, value: T) {
        let shard_idx = self.get_shard(key);
        let mut shard = self.shards[shard_idx].write().unwrap();
        shard.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let shard_idx = self.get_shard(key);
        let shard = self.shards[shard_idx].read().unwrap();
        shard.get(key).cloned()
    }

    pub fn has(&self, key: &str) -> bool {
        let shard_idx = self.get_shard(key);
        let shard = self.shards[shard_idx].read().unwrap();
        shard.contains_key(key)
    }

    pub fn remove(&self, key: &str) -> Option<T> {
        let shard_idx = self.get_shard(key);
        let mut shard = self.shards[shard_idx].write().unwrap();
        shard.remove(key)
    }
}

pub enum CacheSet<T> {
    Insert { key: String, value: T },
    Remove { key: String },
    Get { key: String },
}

impl<T: Clone + 'static> Message for CacheSet<T> {
    type Result = Option<T>;
}

impl<T: Clone + Send + 'static + Unpin> Message for ShardedCache<T> {
    type Result = Option<T>;
}

impl<T: Clone + Send + 'static + Unpin> Actor for ShardedCache<T> {
    type Context = Context<Self>;
}

impl<T: Clone + Send + 'static + Unpin> Handler<CacheSet<T>> for ShardedCache<T> {
    type Result = Option<T>;
    fn handle(&mut self, msg: CacheSet<T>, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            CacheSet::Insert { key, value } => {
                self.set(&key, value);
                None
            }
            CacheSet::Remove { key } => {
                self.remove(&key);
                None
            }
            CacheSet::Get { key } => self.get(&key),
        }
    }
}
