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
| `sort_runs_by_angle` | `n = 300` | `fn half(a: Pt) -> Int` in std/geometry | 770 ms | 640 ms | a leaf whose arms all return (its body ends in lowering's unreachable trap) inlines into `x_angle_less`, with no `CallVmMix` per comparison |
| `match_payload_enum` | `n = 200000`: 200000 masks over a 12-`Slot` list | `acc += (1..q.len).count(i => species(q[i - 1], mask) != species(q[i], mask))` | 2260.0 ms | 1663.5 ms | a payload enum's variant test is `IsVariantI` and its payload a field read, not `ApplyV` (524026262) |
| `sort_runs_by_angle` | `n = 300` | `fn cross128(self, b: Pt) -> I128` in std/geometry | 760 ms | 620 ms | a leaf that returns an `I128` (or `Float`) inlines into a V register, so `x_angle_less` makes no call and is itself a leaf |
