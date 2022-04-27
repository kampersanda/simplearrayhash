# simplearrayhash

![](https://github.com/kampersanda/simplearrayhash/actions/workflows/rust.yml/badge.svg)
[![Documentation](https://docs.rs/simplearrayhash/badge.svg)](https://docs.rs/simplearrayhash)
[![Crates.io](https://img.shields.io/crates/v/simplearrayhash.svg)](https://crates.io/crates/simplearrayhash)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kampersanda/simplearrayhash/blob/master/LICENSE)

Just a fast hash table for string keys.

For example, a benchmark result using `unidic` showed

```
simplearrayhash::HashMap::get = 25.869 us
std::collections::HashMap::get = 104.38 us
```

on my laptop PC.