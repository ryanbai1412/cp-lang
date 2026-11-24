# Interpreter benchmarks

`cpl run` time for shapes that timed out in acceptance, as `cpl profile`
reports them (`cargo build --release -p cpl-cli`, `CPL_STD_DIR=$PWD/std`,
best of 3, one 4-vCPU box). Each row names the change it measures.

```sh
echo 300 | target/release/cpl profile bench/interp/sort_runs_by_angle.cp
```

| bench | input | line | before | after | change |
|---|---|---|---:|---:|---|
| `sort_runs_by_angle` | `n = 300`: 300 rounds of 300 `Pt`s | `ds.runs(by: d => arg(d))` | 381.5 ms | 351.2 ms | `runs(by:)` calls `by` once per element, not again at each run's start |
