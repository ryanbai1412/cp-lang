# Containers Benchmarks

Matched-algorithm benchmarks comparing CP Lang containers against hand-written C++17 (`g++ -O2`).

Run all:
```bash
bash bench/containers/run.sh
```

## Hash maps (`HashMap<Int, Int>` vs `__gnu_pbds::gp_hash_table` with salted `splitmix64`)

| Benchmark | Workload | CP Lang | Hand C++ | Ratio |
|---|---|---:|---:|---:|
| `hash_insert` | 500k random inserts/updates + sum | 51.9 ms | 77.5 ms | 0.67x |
| `hash_lookup` | 200k keys, 2M hit/miss lookups | 71.0 ms | 92.5 ms | 0.77x |
| `hash_antihash` | 300k inserts + 300k lookups (multiples of 126271 & 64) | 44.4 ms | 69.9 ms | 0.64x |
