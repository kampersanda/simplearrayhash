use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use fasthash::{city, RandomState};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, SamplingMode,
};

const SAMPLE_SIZE: usize = 10;
const WARM_UP_TIME: Duration = Duration::from_secs(5);
const MEASURE_TIME: Duration = Duration::from_secs(10);

fn criterion_unidic_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("unidic/get");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(WARM_UP_TIME);
    group.measurement_time(MEASURE_TIME);
    group.sampling_mode(SamplingMode::Flat);
    let mut keys = load_file("data/unidic");
    let queries = load_file("data/unidic.1k.queries");

    keys.sort_unstable();
    add_get_benches(&mut group, &keys, &queries);
}

fn add_get_benches(group: &mut BenchmarkGroup<WallTime>, keys: &[String], queries: &[String]) {
    group.bench_function("simplearrayhash", |b| {
        let records: Vec<_> = keys.iter().enumerate().map(|(i, k)| (k, i)).collect();
        let map = simplearrayhash::HashMap::new(&records).unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += map.get(query).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("std/HashMap", |b| {
        let mut map = std::collections::HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            map.insert(key, i as u32);
        }
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += map.get(query).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("std/HashMap/city", |b| {
        let s = RandomState::<city::Hash64>::new();
        let mut map = std::collections::HashMap::with_hasher(s);
        for (i, key) in keys.iter().enumerate() {
            map.insert(key, i as u32);
        }
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += map.get(query).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });
}

fn load_file<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}

criterion_group!(benches, criterion_unidic_get);
criterion_main!(benches);
