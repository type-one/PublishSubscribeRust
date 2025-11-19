//-----------------------------------------------------------------------------//
// Rust Publish/Subscribe Pattern - Spare time development for fun             //
// (c) 2025 Laurent Lardinois https://be.linkedin.com/in/laurentlardinois      //
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
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::RwLock;
/// Thread-safe dictionary implementation using standard Rust constructs.
#[derive(Debug)]
pub struct SyncDictionary<K, T> {
    dictionary: RwLock<BTreeMap<K, T>>,
}

/// Implementation of the SyncDictionary methods.
impl<K, T> SyncDictionary<K, T>
where
    K: Ord + Hash + Clone, // Ensure K can be used as a key in BTreeMap and cloned
    T: Clone + Debug,      // Ensure T can be cloned and printed
{
    /// Creates a new SyncDictionary.
    pub fn new() -> Self {
        SyncDictionary {
            dictionary: RwLock::new(BTreeMap::new()),
        }
    }

    /// Inserts a key-value pair into the dictionary.
    pub fn insert(&self, key: K, value: T) {
        let mut dict_guard = self.dictionary.write().unwrap();
        dict_guard.insert(key, value);
    }

    /// Retrieves a value associated with the given key.
    pub fn get(&self, key: &K) -> Option<T> {
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

    /// Adds key-value pairs from a BTreeMap to the dictionary.
    pub fn add_btree_collection(&self, other: &BTreeMap<K, T>) {
        let mut dict_guard = self.dictionary.write().unwrap();
        for (key, value) in other.iter() {
            dict_guard.insert(key.clone(), value.clone());
        }
    }

    /// Adds key-value pairs from a HashMap to the dictionary.
    pub fn add_hash_collection(&self, other: &HashMap<K, T>) {
        let mut dict_guard = self.dictionary.write().unwrap();
        for (key, value) in other.iter() {
            dict_guard.insert(key.clone(), value.clone());
        }
    }

    /// Converts the dictionary to a BTreeMap.
    pub fn to_btree_collection(&self) -> BTreeMap<K, T> {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard.clone()
    }

    /// Converts the dictionary to a HashMap.
    pub fn to_hash_collection(&self) -> HashMap<K, T> {
        let dict_guard = self.dictionary.read().unwrap();
        dict_guard
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

/// Implementation of the Default trait for SyncDictionary.
impl<K, T> Default for SyncDictionary<K, T>
where
    K: Ord + Hash + Clone, // Ensure K can be used as a key in BTreeMap and cloned
    T: Clone + Debug,      // Ensure T can be cloned and printed
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
        assert_eq!(btree.get(&"key1".to_string()), Some(&10));
        assert_eq!(btree.get(&"key2".to_string()), Some(&20));
    }

    // basic test for to_hash_collection
    #[test]
    fn test_to_hash_collection() {
        let dict: SyncDictionary<String, i32> = SyncDictionary::new();
        dict.insert("key1".to_string(), 10);
        dict.insert("key2".to_string(), 20);
        let hash_map = dict.to_hash_collection();
        assert_eq!(hash_map.get(&"key1".to_string()), Some(&10));
        assert_eq!(hash_map.get(&"key2".to_string()), Some(&20));
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
