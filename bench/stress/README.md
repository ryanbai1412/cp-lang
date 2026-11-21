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

    cargo test -p cpl-cli --test seeded_bugs -- --nocapture

A new bug is a new pair of files. A stress change that finds a smaller
counterexample updates the `.min`, with the reason in the commit.
