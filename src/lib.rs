//! # simplearrayhash
//!
//! Just a fast hash table for string keys.
#![deny(missing_docs)]

use anyhow::{anyhow, Result};

const MAX_LOAD_FACTOR: f64 = 0.8;
const WORD_BITS: usize = std::mem::size_of::<usize>() * 8;

trait Node {
    fn new(ptr: usize, len: usize) -> Self;
    fn ptr(&self) -> usize;
    fn len(&self) -> usize;
}

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
        Self {
            ptr,
            len,
            val: V::default(),
        }
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

/// Fast hash map implementation for string kyes.
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
    /// - `records`: Sorted list of key-value pairs.
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
    #[inline(always)]
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).is_some()
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline(always)]
    pub fn get<K>(&self, key: K) -> Option<&V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).map(|nd| &nd.val)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline(always)]
    pub fn get_mut<K>(&mut self, key: K) -> Option<&mut V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get_mut(key).map(|nd| &mut nd.val)
    }

    /// Returns the number of elements in the map.
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

#[derive(Clone)]
struct Table<N>
where
    N: Default + Clone + Node,
{
    nodes: Vec<Option<N>>,
    bytes: Vec<u8>,
    capacity_mask: usize,
    num_keys: usize,
}

impl<N> Table<N>
where
    N: Default + Clone + Node,
{
    fn build<K>(keys: &[K]) -> Self
    where
        K: AsRef<[u8]>,
    {
        let num_keys = keys.len();
        let capacity = ceil_two((num_keys as f64 / MAX_LOAD_FACTOR) as usize);
        let capacity_mask = capacity - 1;
        let mut mapping = vec![None; capacity];
        for (i, key) in keys.iter().enumerate() {
            let mut pos = hash_key(key.as_ref()) & capacity_mask;
            while mapping[pos].is_some() {
                pos = (pos + 1) & capacity_mask;
            }
            mapping[pos] = Some(i);
        }

        let mut nodes = vec![None; mapping.len()];
        let mut bytes = vec![];
        for (i, map) in mapping.iter().enumerate() {
            if let Some(j) = map {
                let ptr = bytes.len();
                let key = keys[*j].as_ref();
                bytes.extend_from_slice(key);
                nodes[i] = Some(N::new(ptr, key.len()));
            }
        }
        bytes.shrink_to_fit();
        Self {
            nodes,
            bytes,
            capacity_mask,
            num_keys,
        }
    }

    #[inline(always)]
    fn get<K>(&self, key: K) -> Option<&N>
    where
        K: AsRef<[u8]>,
    {
        self.get_pos(key.as_ref())
            .and_then(|pos| self.nodes[pos].as_ref())
    }

    #[inline(always)]
    fn get_mut<K>(&mut self, key: K) -> Option<&mut N>
    where
        K: AsRef<[u8]>,
    {
        self.get_pos(key.as_ref())
            .and_then(|pos| self.nodes[pos].as_mut())
    }

    #[inline(always)]
    fn get_pos<K>(&self, key: K) -> Option<usize>
    where
        K: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let mut pos = hash_key(key) & self.capacity_mask;
        while let Some(node) = &self.nodes[pos] {
            if key == self.get_bytes(node) {
                return Some(pos);
            }
            pos = (pos + 1) & self.capacity_mask;
        }
        None
    }

    #[inline(always)]
    fn get_bytes(&self, node: &N) -> &[u8] {
        &self.bytes[node.ptr()..node.ptr() + node.len()]
    }

    #[inline(always)]
    #[allow(clippy::missing_const_for_fn)]
    fn num_keys(&self) -> usize {
        self.num_keys
    }
}

#[inline(always)]
fn hash_key(k: &[u8]) -> usize {
    fasthash::city::hash64(k) as usize
}

const fn ceil_two(n: usize) -> usize {
    1 << (WORD_BITS - n.leading_zeros() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let keys = vec!["icdm", "idce", "sigmod", "sigir", "acl"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        let map = HashMap::new(&records).unwrap();
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_get() {
        let keys = vec!["icdm", "idce", "sigmod", "sigir", "acl"];
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
        let keys = vec!["icdm", "idce", "sigmod", "sigir", "acl"];
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
    fn test_get_with_empty_key() {
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
