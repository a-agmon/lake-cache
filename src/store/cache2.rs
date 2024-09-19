use bytes::Bytes;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct CacheItem {
    value: Bytes,
    expiration_time_millis: u128,
}

struct CacheData {
    cache: HashMap<String, CacheItem>,
    cache_index: HashMap<String, usize>,
    keys: VecDeque<String>,
    capacity: usize,
    ttl_seconds: usize,
}

thread_local! {
    static CACHE_DATA: RefCell<CacheData> = RefCell::new(CacheData {
        cache: HashMap::new(),
        cache_index: HashMap::new(),
        keys: VecDeque::new(),
        capacity: 0,
        ttl_seconds: 0,
    });
}

pub struct LRUCache2;

impl LRUCache2 {
    pub fn new(capacity: usize, ttl_seconds: usize) -> Self {
        CACHE_DATA.with(|data| {
            data.borrow_mut().capacity = capacity;
            data.borrow_mut().ttl_seconds = ttl_seconds;
        });
        LRUCache2
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        CACHE_DATA.with(|data| {
            let mut data = data.borrow_mut();
            if let Some(item) = data.cache.get(key).cloned() {
                if item.expiration_time_millis > current_time_millis() {
                    Self::push_item_to_front(&mut data, key);
                    Some(item.value)
                } else {
                    Self::evict_by_key(&mut data, key);
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn set(&self, key: &str, value: Bytes) {
        CACHE_DATA.with(|data| {
            let mut data = data.borrow_mut();
            let item = CacheItem {
                value,
                expiration_time_millis: current_time_millis() + (data.ttl_seconds * 1000) as u128,
            };
            data.cache.insert(key.to_string(), item);
            Self::push_item_to_front(&mut data, key);
            Self::evict_if_necessary(&mut data);
        });
    }

    fn push_item_to_front(data: &mut CacheData, key: &str) {
        if let Some(index) = data.cache_index.get(key) {
            data.keys.remove(*index);
        }
        data.keys.push_front(key.to_string());
        data.cache_index.insert(key.to_string(), 0);
    }

    fn evict_by_key(data: &mut CacheData, key: &str) {
        if let Some(index) = data.cache_index.get(key) {
            data.keys.remove(*index);
            data.cache.remove(key);
            data.cache_index.remove(key);
        }
    }

    fn evict_if_necessary(data: &mut CacheData) {
        if data.keys.len() > data.capacity {
            if let Some(lru_key) = data.keys.pop_back() {
                data.cache.remove(&lru_key);
                data.cache_index.remove(&lru_key);
            }
        }
    }
}

fn current_time_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let cache = LRUCache2::new(2, 5);

        // Test setting and getting values
        cache.set("key1", Bytes::from("value1"));
        cache.set("key2", Bytes::from("value2"));

        assert_eq!(cache.get("key1"), Some(Bytes::from("value1")));
        assert_eq!(cache.get("key2"), Some(Bytes::from("value2")));
        assert_eq!(cache.get("key3"), None);

        // Test capacity and LRU eviction
        cache.set("key3", Bytes::from("value3"));
        assert_eq!(cache.get("key1"), None); // key1 should be evicted
        assert_eq!(cache.get("key2"), Some(Bytes::from("value2")));
        assert_eq!(cache.get("key3"), Some(Bytes::from("value3")));

        // Test updating existing key
        cache.set("key2", Bytes::from("new_value2"));
        assert_eq!(cache.get("key2"), Some(Bytes::from("new_value2")));

        // Test order after access
        cache.get("key3"); // This should move key3 to the front
        cache.set("key4", Bytes::from("value4"));
        assert_eq!(cache.get("key2"), None); // key2 should be evicted
        assert_eq!(cache.get("key3"), Some(Bytes::from("value3")));
        assert_eq!(cache.get("key4"), Some(Bytes::from("value4")));
    }

    #[test]
    fn test_ttl_expiry() {
        let cache = LRUCache2::new(2, 2); // 100ms TTL

        cache.set("key1", Bytes::from("value1"));
        cache.set("key2", Bytes::from("value2"));

        assert_eq!(cache.get("key1"), Some(Bytes::from("value1")));
        assert_eq!(cache.get("key2"), Some(Bytes::from("value2")));

        // Sleep for 1 second (half the TTL)
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Keys should still be present
        assert_eq!(cache.get("key1"), Some(Bytes::from("value1")));
        assert_eq!(cache.get("key2"), Some(Bytes::from("value2")));

        // Sleep for another second (total 2 seconds, exceeding TTL)
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Keys should now be expired
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), None);

        // Test that setting a new key after expiry works
        cache.set("key3", Bytes::from("value3"));
        assert_eq!(cache.get("key3"), Some(Bytes::from("value3")));
    }
}
