# simplearrayhash

Just a fast hash table for string keys.

For example, a benchmark result using `unidic` showed

```
simplearrayhash::HashMap::get = 25.869 us
std::collections::HashMap::get = 104.38 us
```

on my laptop PC.