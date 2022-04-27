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
    pub fn new<K>(records: &[(K, V)]) -> Self
    where
        K: AsRef<[u8]>,
    {
        let keys: Vec<_> = records.iter().map(|(k, _)| k).collect();
        let mut table = Table::<MapNode<V>>::build(&keys);
        let mut flags = vec![false; table.nodes.len()]; // to check duplication
        for (k, v) in records {
            let pos = table.get_pos(k).unwrap();
            assert!(!flags[pos]);
            table.nodes[pos].as_mut().unwrap().val = v.clone();
            flags[pos] = true;
        }
        Self { table }
    }

    #[inline(always)]
    pub fn contains<K>(&self, key: K) -> bool
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).is_some()
    }

    #[inline(always)]
    pub fn get<K>(&self, key: K) -> Option<&V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get(key).map(|nd| &nd.val)
    }

    #[inline(always)]
    pub fn get_mut<K>(&mut self, key: K) -> Option<&mut V>
    where
        K: AsRef<[u8]>,
    {
        self.table.get_mut(key).map(|nd| &mut nd.val)
    }

    #[inline(always)]
    pub fn num_keys(&self) -> usize {
        self.table.num_keys()
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
        for (i, key) in keys.into_iter().enumerate() {
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
    fn num_keys(&self) -> usize {
        self.num_keys
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
        let map = HashMap::new(&records);
        for (k, v) in records {
            assert_eq!(*map.get(k).unwrap(), v);
        }
    }
}
