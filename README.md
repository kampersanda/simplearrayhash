# simplearrayhash

![](https://github.com/kampersanda/simplearrayhash/actions/workflows/rust.yml/badge.svg)
[![Documentation](https://docs.rs/simplearrayhash/badge.svg)](https://docs.rs/simplearrayhash)
[![Crates.io](https://img.shields.io/crates/v/simplearrayhash.svg)](https://crates.io/crates/simplearrayhash)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kampersanda/simplearrayhash/blob/master/LICENSE)

This is a simple fast implementation of a open addressing hash table for string keys.
Its memory layout follows the idea behind of [`array-hash`](https://tessil.github.io/2017/06/22/hat-trie.html#array-hash-table).

For example, a benchmark result that querying 1K Japanese words showed

```
simplearrayhash::HashMap::get = 25.869 us
std::collections::HashMap::get = 104.38 us
```

on my laptop PC. Visit the directory `bench` to see the benchmark setting.