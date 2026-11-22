# Seeded-bug benchmark for `cpl stress`

Each `<port>-<bug>.cp` is a port from `proto/ports/` with one classic bug
planted in it (wrong greedy, sentinel, off-by-one, item reuse, multitest
state leak, ...) and the correct fn as its `@brute`. Its `.min` is the
smallest input that shows the bug: what one `cpl stress <bug>.cp` must
print, with no generator, no flags and nothing else written.

`crates/cpl-cli/tests/seeded_bugs.rs` runs every bug in the gate, prints
the time to the minimal counterexample per bug and the median, and fails
if a bug is missed or shrinks to anything but its `.min`. Each run has a
fixed seed and case count, so the verdict never depends on the machine;
the median is measured against the 1 s bar (`GOALS` 4), not asserted.

`judged.py` times wrong solutions to every judged port (interactor or
checker) to a counterexample: `print(1)` and one mutant per operator, with
p50 and p95 over the caught runs. It is a measurement, not a gate test.

`budget.py` checks the `plan/ARCHITECTURE.md` A5 stress budget (time to a
shrunk counterexample for a false conjecture under 10 s) on 3 planted-bug
problems of different shapes (`@alt(program)`, `@gen` + part `@alt`, and
`@check`). Run it outside the fast gate:

    python3 bench/stress/budget.py

    cargo test -p cpl-cli --test seeded_bugs -- --nocapture

Each bug names its class on a `// class:` line: one of the P0 classes of
`AGENTS.md` goal 2 (`off-by-one`, `index-space`, `overflow`, `boundaries`,
`multitest-leak`, `io-flush`, `sentinel`, `complexity`, `mutation`,
`modulus`, `floating-point`) or `logic` for a wrong algorithm.
`every_seeded_bug_names_its_bug_class` prints the count per class.

A new bug is a new pair of files. A stress change that finds a smaller
counterexample updates the `.min`, with the reason in the commit.
