# compiler

This folder holds the compiler domain's plan: [PLAN.md](PLAN.md) (the work
queue and fixes owed), [metrics.toml](metrics.toml) (the numbers and the
commands that measure them) and [worklog.md](worklog.md) (what landed and
why). What the domain owns, its interfaces and its quality register are in
the charter, [handbook/domains/compiler.md](../../../handbook/domains/compiler.md);
how the compiler is built is in
[handbook/design/compiler.md](../../../handbook/design/compiler.md). Current
status is in `plan/STATUS.md`, generated from the metrics.

## Milestone 1 bar

For Milestone 1 ([handbook/product/milestones.md](../../../handbook/product/milestones.md),
"Language and compiler": correct and identical across interpreter and C++),
the compiler is done when all three hold on main, each measured by its
`milestone = "M1"` row in [metrics.toml](metrics.toml):

- **Conformance 100%:** `cpl-conform conformance` passes every case, with 0
  distinct failure causes.
- **Interpreter == C++, 0 skips:** `cpl-conform diff` reports only `pass`:
  no mismatch, error, `cxx_err` or skip.
- **Tree-walker deleted:** the VM runs every program; 0 fallbacks over the
  programs, corpus and conformance, and the tree-walker interpreter is gone.

A gap that is not yet closed is listed as a known-failing case with its
owner, never disguised.
