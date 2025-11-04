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
use std::sync::Mutex;
/// Thread-safe dictionary implementation using standard Rust constructs.
pub struct SyncDictionary<K, T> {
    dictionary: Mutex<BTreeMap<K, T>>,
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
            dictionary: Mutex::new(BTreeMap::new()),
        }
    }

    /// Inserts a key-value pair into the dictionary.
    pub fn insert(&self, key: K, value: T) {
        let mut dict_guard = self.dictionary.lock().unwrap();
        dict_guard.insert(key, value);
    }

    /// Retrieves a value associated with the given key.
    pub fn get(&self, key: &K) -> Option<T> {
        let dict_guard = self.dictionary.lock().unwrap();
        dict_guard.get(key).cloned()
    }

    /// Removes a key-value pair from the dictionary.
    pub fn remove(&self, key: &K) {
        let mut dict_guard = self.dictionary.lock().unwrap();
        dict_guard.remove(key);
    }

    /// Checks if the dictionary contains a key.
    pub fn contains_key(&self, key: &K) -> bool {
        let dict_guard = self.dictionary.lock().unwrap();
        dict_guard.contains_key(key)
    }

    /// Returns the number of key-value pairs in the dictionary.
    pub fn size(&self) -> usize {
        let dict_guard = self.dictionary.lock().unwrap();
        dict_guard.len()
    }

    /// Clears all key-value pairs from the dictionary.
    pub fn clear(&self) {
        let mut dict_guard = self.dictionary.lock().unwrap();
        dict_guard.clear();
    }

    /// Adds key-value pairs from a BTreeMap to the dictionary.
    pub fn add_btree_collection(&self, other: &BTreeMap<K, T>) {
        let mut dict_guard = self.dictionary.lock().unwrap();
        for (key, value) in other.iter() {
            dict_guard.insert(key.clone(), value.clone());
        }
    }

    /// Adds key-value pairs from a HashMap to the dictionary.
    pub fn add_hash_collection(&self, other: &HashMap<K, T>) {
        let mut dict_guard = self.dictionary.lock().unwrap();
        for (key, value) in other.iter() {
            dict_guard.insert(key.clone(), value.clone());
        }
    }

    /// Converts the dictionary to a BTreeMap.
    pub fn to_btree_collection(&self) -> BTreeMap<K, T> {
        let dict_guard = self.dictionary.lock().unwrap();
        dict_guard.clone()
    }

    /// Converts the dictionary to a HashMap.
    pub fn to_hash_collection(&self) -> HashMap<K, T> {
        let dict_guard = self.dictionary.lock().unwrap();
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
