use bytes::Bytes;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

struct Node {
    key: String,
    value: Bytes,
    prev: Option<Arc<RwLock<Node>>>,
    next: Option<Arc<RwLock<Node>>>,
}

pub struct LRUCache {
    capacity: usize,
    cache: HashMap<String, Arc<RwLock<Node>>>,
    head: Option<Arc<RwLock<Node>>>,
    tail: Option<Arc<RwLock<Node>>>,
}

impl LRUCache {
    pub fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            cache: HashMap::new(),
            head: None,
            tail: None,
        }
    }

    fn get(&mut self, key: &str) -> Option<Bytes> {
        if let Some(node) = self.cache.get(key) {
            let value = node.read().unwrap().value.clone();
            self.move_to_head(Arc::clone(node));
            Some(value)
        } else {
            None
        }
    }

    fn set(&mut self, key: String, value: Bytes) {
        if let Some(node) = self.cache.get(&key) {
            node.write().unwrap().value = value;
            self.move_to_head(Arc::clone(node));
        } else {
            let new_node = Arc::new(RwLock::new(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            }));

            if self.cache.len() >= self.capacity {
                if let Some(tail) = self.tail.take() {
                    let tail_key = tail.read().unwrap().key.clone();
                    self.cache.remove(&tail_key);
                    if let Some(prev) = tail.write().unwrap().prev.take() {
                        prev.write().unwrap().next = None;
                        self.tail = Some(prev);
                    }
                }
            }

            self.cache.insert(key, Arc::clone(&new_node));
            self.add_to_head(new_node);
        }
    }

    fn move_to_head(&mut self, node: Arc<RwLock<Node>>) {
        if Arc::ptr_eq(self.head.as_ref().unwrap(), &node) {
            return;
        }

        let prev = node.write().unwrap().prev.take();
        let next = node.write().unwrap().next.take();

        if let Some(prev) = prev.clone() {
            prev.write().unwrap().next = next.clone();
        }

        if let Some(next) = next {
            next.write().unwrap().prev = prev;
        } else {
            self.tail = prev;
        }

        self.add_to_head(node);
    }

    fn add_to_head(&mut self, node: Arc<RwLock<Node>>) {
        node.write().unwrap().prev = None;
        node.write().unwrap().next = self.head.clone();

        if let Some(head) = self.head.clone() {
            head.write().unwrap().prev = Some(Arc::clone(&node));
        }

        self.head = Some(node);

        if self.tail.is_none() {
            self.tail = self.head.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2);

        // Test setting and getting values
        cache.set("key1".to_string(), Bytes::from("value1"));
        cache.set("key2".to_string(), Bytes::from("value2"));

        assert_eq!(cache.get("key1"), Some(Bytes::from("value1")));
        assert_eq!(cache.get("key2"), Some(Bytes::from("value2")));
        assert_eq!(cache.get("key3"), None);

        // Test capacity and LRU eviction
        cache.set("key3".to_string(), Bytes::from("value3"));
        assert_eq!(cache.get("key1"), None); // key1 should be evicted
        assert_eq!(cache.get("key2"), Some(Bytes::from("value2")));
        assert_eq!(cache.get("key3"), Some(Bytes::from("value3")));

        // Test updating existing key
        cache.set("key2".to_string(), Bytes::from("new_value2"));
        assert_eq!(cache.get("key2"), Some(Bytes::from("new_value2")));

        // Test order after access
        cache.get("key3"); // This should move key3 to the front
        cache.set("key4".to_string(), Bytes::from("value4"));
        assert_eq!(cache.get("key2"), None); // key2 should be evicted
        assert_eq!(cache.get("key3"), Some(Bytes::from("value3")));
        assert_eq!(cache.get("key4"), Some(Bytes::from("value4")));
    }
}
