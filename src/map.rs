//! Simple fast hash map implementation for string kyes.

use crate::{Node, Table};

use anyhow::{anyhow, Result};

#[derive(Default, Clone)]
struct MapNode<V> {
    ptr: usize,
    len: usize,
    val: V,
}

impl<V> Node for MapNode<V>
where
    V: Default,
{
    fn new(ptr: usize, len: usize) -> Self {
        let val = V::default();
        Self { ptr, len, val }
    }

    #[inline(always)]
    fn ptr(&self) -> usize {
        self.ptr
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}

/// Simple fast hash map implementation for string kyes.
#[derive(Clone)]
pub struct HashMap<V>
where
    V: Default + Clone,
{
    table: Table<MapNode<V>>,
}

impl<V> HashMap<V>
where
    V: Default + Clone,
{
    /// Creates a new [`HashMap`] from input records.
    ///
    /// # Arguments
    ///
    /// - `records`: List of key-value pairs.
    ///
    /// # Errors
    ///
    /// An error will be returned when
    ///
    ///  - `records` is empty, or
    ///  - `records` contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashMap;
    ///
    /// let records = vec![("icdm", 0), ("idce", 1), ("sigmod", 2)];
    /// let map = HashMap::new(&records).unwrap();
    /// assert_eq!(map.get("idce"), Some(&1));
    /// assert_eq!(map.get("sigir"), None);
    /// ```
    pub fn new<K>(records: &[(K, V)]) -> Result<Self>
    where
        K: AsRef<[u8]>,
    {
        if records.is_empty() {
            return Err(anyhow!("The input records must not be empty."));
        }
        let keys: Vec<_> = records.iter().map(|(k, _)| k).collect();
        let mut table = Table::<MapNode<V>>::build(&keys);
        let mut flags = vec![false; table.nodes.len()]; // to check duplication
        for (k, v) in records {
            let pos = table.get_pos(k).unwrap();
            if flags[pos] {
                return Err(anyhow!(
                    "The input records must not contain duplicated keys."
                ));
            }
            table.nodes[pos].as_mut().unwrap().val = v.clone();
            flags[pos] = true;
        }
        Ok(Self { table })
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashMap;
    ///
    /// let records = vec![("icdm", 0), ("idce", 1), ("sigmod", 2)];
    /// let map = HashMap::new(&records).unwrap();
    /// assert_eq!(map.contains_key("idce"), true);
    /// assert_eq!(map.contains_key("sigir"), false);
    /// ```
    #[inline(always)]
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).is_some()
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashMap;
    ///
    /// let records = vec![("icdm", 0), ("idce", 1), ("sigmod", 2)];
    /// let map = HashMap::new(&records).unwrap();
    /// assert_eq!(map.get("idce"), Some(&1));
    /// assert_eq!(map.get("sigir"), None);
    /// ```
    #[inline(always)]
    pub fn get<K>(&self, key: K) -> Option<&V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).map(|nd| &nd.val)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashMap;
    ///
    /// let records = vec![("icdm", 0), ("idce", 1), ("sigmod", 2)];
    /// let mut map = HashMap::new(&records).unwrap();
    /// *map.get_mut("idce").unwrap() = 3;
    /// assert_eq!(map.get("idce"), Some(&3));
    /// ```
    #[inline(always)]
    pub fn get_mut<K>(&mut self, key: K) -> Option<&mut V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get_mut(key).map(|nd| &mut nd.val)
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashMap;
    ///
    /// let records = vec![("icdm", 0), ("idce", 1), ("sigmod", 2)];
    /// let map = HashMap::new(&records).unwrap();
    /// assert_eq!(map.len(), 3);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.table.num_keys()
    }

    /// Returns true if the map contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let keys = vec!["icdm", "idce", "", "sigmod", "sigir", "acl"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        let map = HashMap::new(&records).unwrap();
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn test_get() {
        let keys = vec!["icdm", "idce", "", "sigmod", "sigir", "acl"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        let map = HashMap::new(&records).unwrap();
        for &(k, v) in &records {
            assert_eq!(*map.get(k).unwrap(), v);
        }
        assert_eq!(map.get("sigkdd"), None);
        assert_eq!(map.get("idml"), None);
    }

    #[test]
    fn test_get_mut() {
        let keys = vec!["icdm", "idce", "", "sigmod", "sigir", "acl"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        let mut map = HashMap::new(&records).unwrap();
        for &(k, v) in &records {
            *map.get_mut(k).unwrap() = v * 3;
        }
        for &(k, v) in &records {
            assert_eq!(*map.get(k).unwrap(), v * 3);
        }
    }

    #[test]
    #[should_panic]
    fn test_empty() {
        let keys = vec!["icdm"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        HashMap::new(&records[0..0]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_duplicate() {
        let keys = vec!["icdm", "icdm"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        HashMap::new(&records).unwrap();
    }
}
