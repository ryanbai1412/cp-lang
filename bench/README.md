# Max-N Judge & Constant-Factor Audit Harness

This directory holds the Stage-6 worst-case max-N benchmark harness (`max_n_audit.py`) for auditing CP Lang's emitted C++17 against accepted contest C++ originals under judge flags (`g++ -std=c++17 -O2`, no `-DLOCAL`).

## Usage

```sh
# Full 30-port max-N sweep (median of 3 runs) + adversarial probes
python3 bench/max_n_audit.py --runs 3 --adversarial

# Save full JSON metrics
python3 bench/max_n_audit.py --runs 3 --adversarial --json /tmp/max_n_results.json

# Run a subset of ports
python3 bench/max_n_audit.py --ports ioi04p4,noi01p5,hillclimb_dp_graph/dpp --runs 3

# List all 30 ports and their worst-case input shapes
python3 bench/max_n_audit.py --list
```

## What it checks

1. **Differential correctness at max-N:** Copies each `.cp` port to a temp directory (never modifying tracked sources), builds C++17 via `proto/cp.py build`, compiles both emitted and original C++ with `g++ -std=c++17 -O2`, and verifies token-for-token stdout equality on deterministic worst-case contest inputs.
2. **Wall-clock runtime & peak RSS:** Measures median wall time and peak `ru_maxrss` via `/usr/bin/time`, flagging regressions `> 1.25x` runtime (`[SLOW]`) or `> 1.50x` peak RSS (`[MEM]`).
3. **Emitted C++17 file size:** Verifies every generated `.cpp` file stays under the 64 KB judge ceiling.
4. **Adversarial probes (`--adversarial`):**
   - `CpHash` anti-hash splitmix64 inversion (`N = 50,000` keys colliding into bucket 0).
   - `dpp` 100,000-node chain stack-overflow check under `ulimit -s 8192` vs `unlimited`.
   - `CpView`/`cp_win` slice bounds-check microbenchmark (`2e8` ops).
   - `cp_emod` Euclidean modulo microbenchmark (`5e7` ops).

See [`research/max-n-judge-audit-1.md`](../research/max-n-judge-audit-1.md) for the full audit write-up and optimization roadmap.
