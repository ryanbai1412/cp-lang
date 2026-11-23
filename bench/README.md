# Benchmark harnesses

The `rust_*` scripts audit the Rust `cpl` binary.

## Rust `cpl` audits

```sh
cargo build --release -p cpl-cli

# Toolchain latency against the P0 budgets: check, lsp, run, build, stress
python3 bench/rust_latency_audit.py --reps 7

# 40 ports at judge limits against their original C++: output, ASan/UBSan,
# time ratio, peak RSS ratio, emitted size
python3 bench/rust_max_n_audit.py --runs 5
python3 bench/rust_max_n_audit.py --ports dpl,noi22p5 --json /tmp/out.json
python3 bench/rust_max_n_audit.py --list
python3 bench/rust_max_n_audit.py --ecnerwala               # add the ecnerwala ports (sizes assumed)
```

Both use `target/release/cpl` (override with `CPL=/path/to/cpl`), work in a
temp directory and never write to tracked files. `rust_max_n_audit.py` exits
non-zero if any port fails, so it can gate CI. To add a port, append a row to
`SPECS` with a generator `gen(path, big, ctx)` that writes the judge-limit
input when `big` is true and a medium one for the sanitizer run otherwise.
Results and findings: `research/perf/max-n-judge-audit-2-rust.md`.

## Corpus p95 benchmark

`corpus_p95_bench.py` tracks the P0 performance target: the p95 runtime ratio
of emitted C++17 over the original, both built with `g++ -std=c++17 -O2`,
under 1.1x across 200+ corpus ports, with every emitted file under 64 KB.

```sh
cargo build --release -p cpl-cli
python3 bench/corpus_p95_bench.py                     # full sweep, about 17 minutes
python3 bench/corpus_p95_bench.py --ports dpl,dpn --no-write
python3 bench/corpus_p95_bench.py --tail 25 --no-write   # the 25 slowest in the committed results
python3 bench/corpus_p95_bench.py --check             # exit 1 unless the target is met
```

It takes one port per problem from `corpus-500-registry.json`, builds it and
its original, and times both on a maximum-size input where their outputs
match: the judge-limit generators of `rust_max_n_audit.py` where one exists,
otherwise the worst cases `cpl bench` derives from the port's typed `read()`s.
A full sweep rewrites two files:

- [`corpus-p95-report.md`](../research/perf/corpus-p95-report.md): median, p90, p95, p99 and
  worst runtime and peak-RSS ratios, the largest emitted file, the top 20
  runtime offenders, and what kept each unmeasured problem out.
- `corpus-p95-results.json`: every candidate port, with its numbers or the
  reason it was skipped.

The baseline in the report is CP Lang 0.2.2 at `cb8979e`: 94 ports measured,
runtime p95 3.94x, peak-RSS p95 3.46x, largest emitted file 28.5 KB. The
target is not met on either count. To raise the port count, add a generator
to `SPECS` in `rust_max_n_audit.py` for a port the report lists under
`needs a hand-written generator`; the sweep picks it up.

## Trees (`bench/trees/`)

`std.tree` against the same algorithms written directly in C++: rooting with
heavy-first preorder, `reroot` heights, one million `lca` + `dist` queries and
a centroid decomposition read through `ancestors`.

```sh
python3 bench/trees/gen.py 1000000 1000000 1 > big.in   # seed 2: long paths
cpl build bench/trees/tree_bench.cp -o cp.cpp && g++ -std=c++17 -O2 -o cp cp.cpp
g++ -std=c++17 -O2 -o cxx bench/trees/tree_bench.cpp
/usr/bin/time -f '%es %MKB' ./cp < big.in; /usr/bin/time -f '%es %MKB' ./cxx < big.in
```

`root_bench` measures building a `Rooted` alone the same way.
Results and the remaining gap: `bench/trees/README.md`.

## Corpus registry

`corpus_registry.py` is the deduplicated scoreboard over every port in `proto/ports/`: how many
`.cp` files there are against how many distinct problems they solve, which port is the canonical
one for each problem, raw and library-stripped line counts, which ports a harness actually
verified, and which hard corpus problems are still unported.

```
python3 bench/corpus_registry.py                  # rewrite the registry and the report (about a second)
python3 bench/corpus_registry.py --run-harnesses  # first re-run every batch harness (about 15 minutes)
python3 bench/corpus_registry.py --check          # exit 1 if the committed outputs are stale
```

It writes `corpus-500-report.md` (read this one), `corpus-500-registry.json` (every port, problem
and aggregate) and, with `--run-harnesses`, `corpus-500-harness-runs.json` (each harness's exit
code and the line behind every pass or fail). Verification is never inferred: a port added after
the last recorded harness run shows as `not run` until the harnesses are run again.

## Graph searches

`python3 bench/graph/run.py` compares `std.graph` searches against matched
hand-written C++; results and analysis in `bench/graph/README.md`.

## Sequences (`bench/seq/`)

`bash bench/seq/run.sh` compares `std.seq` scans and rotations (`scan`,
`suffix_scan`, `scan_in_place`, `suffix_scan_in_place`, `rotate_left`,
`rotate_right`) against matched C++ `std::partial_sum` and `std::rotate`;
results in `bench/seq/README.md`.
## Math (`bench/math/`)

`bash bench/math/run.sh` compares `std.mod`, `std.ntt`, `std.math`, and
`std.matrix` against matched C++17 (same algorithm and layout, best of 5 runs,
exact stdout match, and emitted C++ byte counts): `modint` arithmetic,
`ntt` convolution (`n = 262,144`), linear `sieve` (`n = 5,000,000` with
`phi_table` + `mobius_table`), `matpow` (`80 x 80` `Mod` matrix to `10^18`),
`crt` (`1,000,000` calls), and `comb` (`limit = 2,000,000`, `5,000,000`
`choose` queries). Results and codegen profiling notes live in
`spec/lib/math.md` and `research/frictions/bench-math.md`.

## Prefix sums and difference arrays (`bench/prefix/`)

`python3 bench/prefix/run.py` compares `Prefix` 1D/2D build+query, `Diff` 1D/2D,
and `FillPrefix` 1D/2D forward/reverse against matched C++17 (`g++ -std=c++17 -O2`,
same layout, best of 5, emitted bytes); results in `bench/prefix/README.md`
(**p50 = 1.10x**, **max = 1.35x**; codegen-only gaps >1.1x logged in
`research/frictions/prefix_bench.md`).
