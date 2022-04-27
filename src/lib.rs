const MAX_LOAD_FACTOR: f64 = 0.8;
const WORD_BITS: usize = std::mem::size_of::<usize>() * 8;

trait Node {
    fn ptr(&self) -> usize;
    fn len(&self) -> usize;
}

#[derive(Default, Clone)]
struct MapNode<V> {
    ptr: usize,
    len: usize,
    val: V,
}

impl<V> Node for MapNode<V> {
    #[inline(always)]
    fn ptr(&self) -> usize {
        self.ptr
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone)]
pub struct Map<V>
where
    V: Default + Clone,
{
    table: Table<MapNode<V>>,
}

impl<V> Map<V>
where
    V: Default + Clone,
{
    pub fn new<K>(records: &[(K, V)]) -> Self
    where
        K: AsRef<[u8]>,
    {
        let keys = records.iter().map(|(k, _)| k);
        let mapping = Table::<MapNode<V>>::build_mapping(keys, records.len());
        let table = Table::set_nodes(&mapping, |i: usize, ptr: usize| {
            let key = records[i].0.as_ref();
            let val = records[i].1.clone();
            let len = key.len();
            (key, MapNode { ptr, len, val })
        });
        Self { table }
    }

    #[inline(always)]
    pub fn contains<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        self.table.contains(key)
    }

    #[inline(always)]
    pub fn get<K>(&self, key: K) -> Option<&V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).map(|nd| &nd.val)
    }
}

#[derive(Clone)]
struct Table<N>
where
    N: Default + Clone + Node,
{
    table: Vec<Option<N>>,
    bytes: Vec<u8>,
    capacity_mask: usize,
}

impl<N> Table<N>
where
    N: Default + Clone + Node,
{
    fn build_mapping<I, K>(keys: I, num_keys: usize) -> Vec<Option<usize>>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<[u8]>,
    {
        let capacity = ceil_two((num_keys as f64 / MAX_LOAD_FACTOR) as usize);
        let capacity_mask = capacity - 1;
        let mut mapping = vec![None; capacity];
        for (i, key) in keys.into_iter().enumerate() {
            let mut pos = hash_key(key.as_ref()) & capacity_mask;
            while mapping[pos].is_some() {
                pos = (pos + 1) & capacity_mask;
            }
            mapping[pos] = Some(i);
        }
        mapping
    }

    fn set_nodes<'a, F>(mapping: &[Option<usize>], getter: F) -> Self
    where
        F: Fn(usize, usize) -> (&'a [u8], N),
    {
        let mut table = vec![None; mapping.len()];
        let mut bytes = vec![];
        for (i, map) in mapping.iter().enumerate() {
            if let Some(j) = map {
                let ptr = bytes.len();
                let (key, node) = getter(*j, ptr);
                bytes.extend_from_slice(key);
                table[i] = Some(node);
            }
        }
        bytes.shrink_to_fit();
        Self {
            table,
            bytes,
            capacity_mask: mapping.len() - 1,
        }
    }

    #[inline(always)]
    fn contains<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let mut pos = hash_key(key) & self.capacity_mask;
        while let Some(node) = &self.table[pos] {
            if key == self.get_bytes(node) {
                return true;
            }
            pos = (pos + 1) & self.capacity_mask;
        }
        false
    }

    #[inline(always)]
    fn get<K>(&self, key: K) -> Option<&N>
    where
        K: AsRef<[u8]>,
    {
        let key = key.as_ref();
        let mut pos = hash_key(key) & self.capacity_mask;
        while let Some(node) = &self.table[pos] {
            if key == self.get_bytes(node) {
                return Some(node);
            }
            pos = (pos + 1) & self.capacity_mask;
        }
        None
    }

    #[inline(always)]
    fn get_bytes(&self, node: &N) -> &[u8] {
        &self.bytes[node.ptr()..node.ptr() + node.len()]
    }
}

#[inline(always)]
fn hash_key(k: &[u8]) -> usize {
    fasthash::city::hash64(k) as usize
}

fn ceil_two(n: usize) -> usize {
    1 << (WORD_BITS - n.leading_zeros() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toy() {
        let keys = vec!["aaa", "abc", "asdddfsb", "adsfv"];
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        let map = Map::new(&records);
        for (k, v) in records {
            assert_eq!(*map.get(k).unwrap(), v);
        }
    }
}
