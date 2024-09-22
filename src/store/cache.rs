use bytes::Bytes;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::{Duration, SystemTime};
thread_local! {
    static CACHE: RefCell<LRUCache> = RefCell::new(LRUCache::new(1, 0));
}
pub struct LocalCache;

impl LocalCache {
    pub fn new(capacity: usize, ttl: u64) -> Self {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            *cache = LRUCache::new(capacity, ttl);
        });
        LocalCache
    }
    pub fn get_item(&self, key: &String) -> Option<Bytes> {
        CACHE.with(|cache| cache.borrow_mut().get_item(key))
    }

    pub fn add_item(&self, key: String, value: Bytes) {
        CACHE.with(|cache| cache.borrow_mut().add_item(key, value))
    }
}

struct Node {
    key: String,
    value: Bytes,
    expires_at: u64,
    prev: Option<Weak<RefCell<Node>>>,
    next: Option<Rc<RefCell<Node>>>,
}

struct LRUCache {
    capacity: usize,
    ttl_seconds: u64,
    map: HashMap<String, Rc<RefCell<Node>>>,
    head: Option<Rc<RefCell<Node>>>,
    tail: Option<Rc<RefCell<Node>>>,
}

impl LRUCache {
    /// Creates a new LRUCache with the given capacity.
    fn new(capacity: usize, ttl_seconds: u64) -> Self {
        LRUCache {
            capacity,
            ttl_seconds,
            map: HashMap::new(),
            head: None,
            tail: None,
        }
    }

    /// Adds an item to the cache. If the item already exists, it updates the value and moves it to the front.
    /// If adding the new item exceeds the capacity, it removes the least recently used item.
    fn add_item(&mut self, key: String, value: Bytes) {
        if let Some(node) = self.map.get(&key) {
            // Update the value and move the node to the head.
            node.borrow_mut().value = value.clone();
            self.move_to_head(Rc::clone(node));
        } else {
            // Create a new node.
            let new_node = Rc::new(RefCell::new(Node {
                key: key.clone(),
                value: value.clone(),
                expires_at: SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + self.ttl_seconds,
                prev: None,
                next: None,
            }));

            // Add the new node to the front and insert it into the map.
            self.add_to_head(Rc::clone(&new_node));
            self.map.insert(key.clone(), Rc::clone(&new_node));

            // If capacity is exceeded, remove the least recently used item.
            if self.map.len() > self.capacity {
                if let Some(tail_node) = self.tail.take() {
                    let tail_key = tail_node.borrow().key.clone();
                    self.remove_node(Rc::clone(&tail_node));
                    self.map.remove(&tail_key);
                }
            }
        }
    }

    /// Retrieves an item from the cache by key. If the item exists, it moves it to the front.
    fn get_item(&mut self, key: &String) -> Option<Bytes> {
        if let Some(node) = self.map.get(key) {
            let value = node.borrow().value.clone();
            self.move_to_head(Rc::clone(node));
            Some(value)
        } else {
            None
        }
    }

    /// Moves the given node to the front of the list.
    fn move_to_head(&mut self, node: Rc<RefCell<Node>>) {
        self.remove_node(Rc::clone(&node));
        self.add_to_head(node);
    }

    /// Removes the given node from the list.
    fn remove_node(&mut self, node: Rc<RefCell<Node>>) {
        let prev_weak = node.borrow_mut().prev.take();
        let next_opt = node.borrow_mut().next.take();

        if let Some(ref prev_weak_ref) = prev_weak {
            if let Some(prev_rc) = prev_weak_ref.upgrade() {
                prev_rc.borrow_mut().next = next_opt.clone();
            }
        } else {
            // Node is head
            self.head = next_opt.clone();
        }

        if let Some(next_rc) = next_opt {
            next_rc.borrow_mut().prev = prev_weak.clone();
        } else {
            // Node is tail
            if let Some(ref prev_weak_ref) = prev_weak {
                self.tail = prev_weak_ref.upgrade();
            } else {
                // List is empty
                self.tail = None;
            }
        }
    }

    /// Adds the given node to the front of the list.
    fn add_to_head(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow_mut().prev = None;
        node.borrow_mut().next = self.head.clone();

        if let Some(old_head) = &self.head {
            old_head.borrow_mut().prev = Some(Rc::downgrade(&node));
        } else {
            // List was empty, so tail is also node
            self.tail = Some(Rc::clone(&node));
        }

        self.head = Some(node);
    }
}
