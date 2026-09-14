//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025-2026 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois //
//                                                                             //
// https://github.com/type-one/PublishSubscribeRust                            //
//                                                                             //
// MIT License                                                                 //
//                                                                             //
// This software is provided 'as-is', without any express or implied           //
// warranty.In no event will the authors be held liable for any damages        //
// arising from the use of this software.                                      //
//                                                                             //
// Permission is granted to anyone to use this software for any purpose,       //
// including commercial applications, and to alter itand redistribute it       //
// freely, subject to the following restrictions :                             //
//                                                                             //
// 1. The origin of this software must not be misrepresented; you must not     //
// claim that you wrote the original software.If you use this software         //
// in a product, an acknowledgment in the product documentation would be       //
// appreciated but is not required.                                            //
// 2. Altered source versions must be plainly marked as such, and must not be  //
// misrepresented as being the original software.                              //
// 3. This notice may not be removed or altered from any source distribution.  //
//-----------------------------------------------------------------------------//

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::RwLock;

/// Minimal associative-container abstraction so `SyncDictionary` can be
/// backed by `BTreeMap`, `HashMap`, or any other compatible container,
/// mirroring the C++ `sync_dictionary<K, T, TDictionary>` template parameter.
pub trait DictionaryContainer<K, T>: Default {
    fn insert(&mut self, key: K, value: T);
    fn get(&self, key: &K) -> Option<&T>;
    fn remove(&mut self, key: &K);
    fn contains_key(&self, key: &K) -> bool;
    fn len(&self) -> usize;
    fn clear(&mut self);
    fn to_vec(&self) -> Vec<(K, T)>
    where
        K: Clone,
        T: Clone;
}

impl<K: Ord, T> DictionaryContainer<K, T> for BTreeMap<K, T> {
    fn insert(&mut self, key: K, value: T) {
        BTreeMap::insert(self, key, value);
    }

    fn get(&self, key: &K) -> Option<&T> {
        BTreeMap::get(self, key)
    }

    fn remove(&mut self, key: &K) {
        BTreeMap::remove(self, key);
    }

    fn contains_key(&self, key: &K) -> bool {
        BTreeMap::contains_key(self, key)
    }

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn clear(&mut self) {
        BTreeMap::clear(self);
    }

    fn to_vec(&self) -> Vec<(K, T)>
    where
        K: Clone,
        T: Clone,
    {
        self.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }
}

impl<K: Eq + Hash, T> DictionaryContainer<K, T> for HashMap<K, T> {
    fn insert(&mut self, key: K, value: T) {
        HashMap::insert(self, key, value);
    }

    fn get(&self, key: &K) -> Option<&T> {
        HashMap::get(self, key)
    }

    fn remove(&mut self, key: &K) {
        HashMap::remove(self, key);
    }

    fn contains_key(&self, key: &K) -> bool {
        HashMap::contains_key(self, key)
    }

    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn clear(&mut self) {
        HashMap::clear(self);
    }

    fn to_vec(&self) -> Vec<(K, T)>
    where
        K: Clone,
        T: Clone,
    {
        self.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }
}

/// Thread-safe dictionary implementation, generic over its backing
/// associative container (`BTreeMap` by default, or `HashMap`, ...).
#[derive(Debug)]
pub struct SyncDictionary<K, T, Container = BTreeMap<K, T>> {
    dictionary: RwLock<Container>,
    _marker: PhantomData<(K, T)>,
}

/// Implementation of the SyncDictionary methods.
impl<K, T, Container> SyncDictionary<K, T, Container>
where
    Container: DictionaryContainer<K, T>,
{
    /// Creates a new, empty SyncDictionary.
    pub fn new() -> Self {
        SyncDictionary {
            dictionary: RwLock::new(Container::default()),
            _marker: PhantomData,
        }
    }

    /// Inserts a key-value pair into the dictionary.
    pub fn insert(&self, key: K, value: T) {
        let mut dict_guard = self.dictionary.write().unwrap();
        dict_guard.insert(key, value);
    }

    /// Retrieves a value associated with the given key.
    pub fn get(&self, key: &K) -> Option<T>
    where
        T: Clone,
    {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.get(key).cloned()
    }

    /// Removes a key-value pair from the dictionary.
    pub fn remove(&self, key: &K) {
        let mut dict_guard = self.dictionary.write().unwrap();
        dict_guard.remove(key);
    }

    /// Checks if the dictionary contains a key.
    pub fn contains_key(&self, key: &K) -> bool {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.contains_key(key)
    }

    /// Checks if the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Returns the number of key-value pairs in the dictionary.
    pub fn size(&self) -> usize {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.len()
    }

    /// Clears all key-value pairs from the dictionary.
    pub fn clear(&self) {
        let mut dict_guard = self.dictionary.write().unwrap();
        dict_guard.clear();
    }

    /// Inserts key-value pairs from an iterator under a single lock.
    /// Returns the number of entries inserted.
    pub fn add_range<I: IntoIterator<Item = (K, T)>>(&self, entries: I) -> usize {
        let mut dict_guard = self.dictionary.write().unwrap();
        let mut count = 0;
        for (key, value) in entries {
            dict_guard.insert(key, value);
            count += 1;
        }
        count
    }

    /// Adds key-value pairs from a BTreeMap to the dictionary.
    pub fn add_btree_collection(&self, other: &BTreeMap<K, T>)
    where
        K: Clone,
        T: Clone,
    {
        self.add_range(
            other
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    /// Adds key-value pairs from a HashMap to the dictionary.
    pub fn add_hash_collection(&self, other: &HashMap<K, T>)
    where
        K: Clone,
        T: Clone,
    {
        self.add_range(
            other
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    /// Returns a clone of the internal backing container.
    pub fn snapshot(&self) -> Container
    where
        Container: Clone,
    {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.clone()
    }

    /// Converts the dictionary to a BTreeMap.
    pub fn to_btree_collection(&self) -> BTreeMap<K, T>
    where
        K: Ord + Clone,
        T: Clone,
    {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.to_vec().into_iter().collect()
    }

    /// Converts the dictionary to a HashMap.
    pub fn to_hash_collection(&self) -> HashMap<K, T>
    where
        K: Eq + Hash + Clone,
        T: Clone,
    {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.to_vec().into_iter().collect()
    }
}

/// Implementation of the Default trait for SyncDictionary.
impl<K, T, Container> Default for SyncDictionary<K, T, Container>
where
    Container: DictionaryContainer<K, T>,
{
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for SyncDictionary.
#[cfg(test)]
mod tests {
    use super::SyncDictionary;
    use std::collections::{BTreeMap, HashMap};

    // basic test for insert and get
    #[test]
    fn test_insert_get() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        assert_eq!(dict.get(&"key1".to_string()), Some(10));
    }

    // basic test for remove
    #[test]
    fn test_remove() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        dict.remove(&"key1".to_string());
        assert_eq!(dict.get(&"key1".to_string()), None);
    }

    // basic test for contains_key
    #[test]
    fn test_contains_key() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        assert!(dict.contains_key(&"key1".to_string()));
        assert!(!dict.contains_key(&"key2".to_string()));
    }

    // basic test for size and clear
    #[test]
    fn test_size_clear() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        dict.insert("key2".to_string(), 20);
        assert_eq!(dict.size(), 2);
        dict.clear();
        assert_eq!(dict.size(), 0);
    }

    // basic test for is_empty
    #[test]
    fn test_is_empty() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        assert!(dict.is_empty());
        dict.insert("key1".to_string(), 10);
        assert!(!dict.is_empty());
    }

    // basic test for add_range batch insertion
    #[test]
    fn test_add_range() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        let inserted = dict.add_range(vec![("key1".to_string(), 10), ("key2".to_string(), 20)]);
        assert_eq!(inserted, 2);
        assert_eq!(dict.get(&"key1".to_string()), Some(10));
        assert_eq!(dict.get(&"key2".to_string()), Some(20));
    }

    // test that the backing container can be swapped for a HashMap
    #[test]
    fn test_hash_map_backed_dictionary() {
        let dict: SyncDictionary<String, i32, HashMap<String, i32>> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        assert_eq!(dict.get(&"key1".to_string()), Some(10));
        assert_eq!(dict.size(), 1);
    }

    // basic test for add_btree_collection
    #[test]
    fn test_add_btree_collection() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        let mut btree = BTreeMap::new();
        btree.insert("key1".to_string(), 10);
        btree.insert("key2".to_string(), 20);
        dict.add_btree_collection(&btree);
        assert_eq!(dict.get(&"key1".to_string()), Some(10));
        assert_eq!(dict.get(&"key2".to_string()), Some(20));
    }

    // basic test for add_hash_collection
    #[test]
    fn test_add_hash_collection() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        let mut hash_map = HashMap::new();
        hash_map.insert("key1".to_string(), 10);
        hash_map.insert("key2".to_string(), 20);
        dict.add_hash_collection(&hash_map);
        assert_eq!(dict.get(&"key1".to_string()), Some(10));
        assert_eq!(dict.get(&"key2".to_string()), Some(20));
    }

    // basic test for to_btree_collection
    #[test]
    fn test_to_btree_collection() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        dict.insert("key2".to_string(), 20);
        let btree = dict.to_btree_collection();
        assert_eq!(btree.get("key1"), Some(&10));
        assert_eq!(btree.get("key2"), Some(&20));
    }

    // basic test for to_hash_collection
    #[test]
    fn test_to_hash_collection() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        dict.insert("key2".to_string(), 20);
        let hash_map = dict.to_hash_collection();
        assert_eq!(hash_map.get("key1"), Some(&10));
        assert_eq!(hash_map.get("key2"), Some(&20));
    }

    // Additional test with two threads
    #[test]
    fn test_concurrent_insert_get() {
        use std::sync::Arc;
        use std::thread;

        let dict: Arc<SyncDictionary<String, i32>> = Arc::new(SyncDictionary::new());
        let dict_for_inserter = dict.clone();
        let dict_for_getter = dict.clone();

        let inserter = thread::spawn(move || {
            for i in 0..100 {
                dict_for_inserter.insert(format!("key{}", i), i);
            }
        });

        let getter = thread::spawn(move || {
            for i in 0..100 {
                loop {
                    if let Some(value) = dict_for_getter.get(&format!("key{}", i)) {
                        assert_eq!(value, i);
                        break;
                    }
                }
            }
        });

        inserter.join().unwrap();
        getter.join().unwrap();
    }

    // test Default trait
    #[test]
    fn test_default() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::default();
        assert_eq!(dict.size(), 0);
    }
}
