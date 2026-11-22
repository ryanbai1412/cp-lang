# compiler

Segment: B. Metric. Owns: `crates/cpl-syntax`, `cpl-check`, `cpl-ir`, `cpl-interp`, `prover` (proofs, range analysis). Worklog: [worklog.md](worklog.md). Architecture: [ARCHITECTURE.md](ARCHITECTURE.md).

## Covers
Parser, checker, IR, interpreter, prover (proofs and range analysis): everything that makes the spec's programs run, in both backends.

## Improve

- The open bug list: product domains/positions, domain `Seq` methods, top-level `let ... else`, `let Some(y) = x else`, explicit `SegTree` type args, fieldless enums as domains, allocation-free enum iteration, dense domain-keyed `tally`, subsets, `Action.compose` order, prefix dimensionality, bare `+` as a value, `todo` in value-returning fns.
- Interpreter vs C++ disagreements (~100 programs).
- Strict mode: reject what the spec doesn't define.

## Success criteria
`cpl-conform all --check` 100% (today 1617/1711); interpreter == C++ on every program (today 1242/1336), tracked as its own number; every fix lands with the case that exposed it.

## Ryan's role
Signs off the targets; reviews only architecture calls (one VM, how domains/products lower, strict mode), via the decision queue.
