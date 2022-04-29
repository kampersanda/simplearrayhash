//! Simple fast hash set implementation for string kyes.

use crate::{Node, Table};

use anyhow::{anyhow, Result};

#[derive(Default, Clone)]
struct SetNode {
    ptr: usize,
    len: usize,
}

impl Node for SetNode {
    fn new(ptr: usize, len: usize) -> Self {
        Self { ptr, len }
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

/// Simple fast hash set implementation for string kyes.
#[derive(Clone)]
pub struct HashSet {
    table: Table<SetNode>,
}

impl HashSet {
    /// Creates a new [`HashSet`] from input keys.
    ///
    /// # Arguments
    ///
    /// - `keys`: List of keys.
    ///
    /// # Errors
    ///
    /// An error will be returned when
    ///
    ///  - `keys` is empty, or
    ///  - `keys` contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashSet;
    ///
    /// let keys = vec!["icdm", "idce", "sigmod"];
    /// let set = HashSet::new(&keys).unwrap();
    /// assert!(set.contains("idce"));
    /// assert!(!set.contains("sigir"));
    /// ```
    pub fn new<K>(keys: &[K]) -> Result<Self>
    where
        K: AsRef<[u8]>,
    {
        if keys.is_empty() {
            return Err(anyhow!("The input keys must not be empty."));
        }
        let table = Table::<SetNode>::build(&keys);
        let mut flags = vec![false; table.nodes.len()]; // to check duplication
        for k in keys {
            let pos = table.get_pos(k).unwrap();
            if flags[pos] {
                return Err(anyhow!("The input keys must not be duplicated."));
            }
            flags[pos] = true;
        }
        Ok(Self { table })
    }

    /// Returns true if the set contains a key.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashSet;
    ///
    /// let keys = vec!["icdm", "idce", "sigmod"];
    /// let set = HashSet::new(&keys).unwrap();
    /// assert!(set.contains("idce"));
    /// assert!(!set.contains("sigir"));
    /// ```
    #[inline(always)]
    pub fn contains<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).is_some()
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use simplearrayhash::HashSet;
    ///
    /// let keys = vec!["icdm", "idce", "sigmod"];
    /// let set = HashSet::new(&keys).unwrap();
    /// assert_eq!(set.len(), 3);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.table.num_keys()
    }

    /// Returns true if the set contains no elements.
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
        let keys = vec!["icdm", "idce", "sigmod", "sigir", "acl"];
        let set = HashSet::new(&keys).unwrap();
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_get() {
        let keys = vec!["icdm", "idce", "sigmod", "sigir", "acl"];
        let set = HashSet::new(&keys).unwrap();
        for &k in &keys {
            assert!(set.contains(k));
        }
        assert!(!set.contains("sigkdd"));
        assert!(!set.contains("idml"));
    }

    #[test]
    fn test_get_with_empty_key() {
        let keys = vec!["icdm", "idce", "", "sigmod", "sigir", "acl"];
        let set = HashSet::new(&keys).unwrap();
        for &k in &keys {
            assert!(set.contains(k));
        }
        assert!(!set.contains("sigkdd"));
        assert!(!set.contains("idml"));
    }

    #[test]
    #[should_panic]
    fn test_empty() {
        let keys = vec!["icdm"];
        HashSet::new(&keys[0..0]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_duplicate() {
        let keys = vec!["icdm", "icdm"];
        HashSet::new(&keys).unwrap();
    }
}
