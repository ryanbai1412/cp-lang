# perf worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Note from infra: trivial unblock in `crates/cpl-codegen/src/emit.rs`: the fixed include list now has `#include <utility>`, since `std::exchange` (`Mutation::Take`) failed to compile on GitHub runners' libstdc++ (CI run 45067797421: 6 of 1116 tests). Guarded by `seq_take_is_o1_move_without_copy`. Revert that commit if you prefer emitting it only when used.
- Request from tech-debt (cold review round 1 of the packet, 986a3e9f; rubric right home 3, one way 2, local next change 3, legible 5, finished migration 3). Every path, line count (`wc -l crates/cpl-codegen/src/*.rs`, 16280 total), consumer and the 91 shape tests check out; `bench/rust_latency_audit.py --help` fails with `ModuleNotFoundError: No module named 'max_n_audit'`, as the packet says.
  1. Two writers of one file: `bench/corpus_p95_bench.py` and `crates/cpl-contest/src/corpus_bench.rs` both write `bench/corpus-p95-results.json`, which `web/lib/bench.ts` reads with no schema test. Keep the Rust runner, delete the Python one (and `rust_max_n_audit.py`/`rust_latency_audit.py` once their suites are `cpl bench` suites), and pin the schema with a test in `crates/cpl-contest`.
  2. Ownership split: `plan/DOMAINS.md` gives `cpl-contest` to tools and `cpl bench` to perf, so `bench.rs`, `zero_cost.rs` and `corpus_bench.rs` have two owners. Proposal for the coordinator: list those three files as perf's in DOMAINS.md; no new crate (a `cpl-bench` crate adds shipped edges the ratchet would flag).
  3. `map_remove_do` (`src/expr.rs:2796`) matching lowered blocks: prefer an explicit IR node from compiler over contract tests, so the fast path cannot silently vanish (one mechanism, not a recognizer plus a test that it still recognizes).
  4. `src/stmt.rs` fusion landing as `src/fuse.rs` matches the file-size ratchet (a new file over 2000 lines fails; `stmt.rs` is not allowlisted).
  5. A perf gate: one `cpl bench --zero-cost` smoke in `scripts/check.sh` (the 1.05x bar on a fixed small set) would turn the "regress silently" debt into a ratchet.
- Landed devin/wip-codegen-perf, split so `stmt.rs` stays under the cap: `bench/v03_perf_bench.py`, v0.3 fixes to zero-cost `modular` and `segtree_prefix`, loop fusion in `crates/cpl-codegen/src/fuse.rs` (now also requires one side trap-free so traps keep their order), power-of-two `/` `%` in `src/divrem.rs`. `cpl bench --zero-cost` (noisy box): 3/7 PASS, both previously broken pairs build (modular 0.98x, segtree_prefix 0.88x vs hand C++), iterators 7.98x -> 1.28x vs hand C++ (0.81x vs direct). Next: M1 perf set.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE; three questions in plan/QUESTIONS.md. Measured on main: zero-cost 3/7, perf set p50 0.987x max 2.431x (3 over 1.5x), v0.3 C++ max 21.7 KB. Next: phase 2 step 1 (land devin/wip-codegen-perf).
- Lane opened.
