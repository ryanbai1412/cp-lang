# perf

Segment: B. Metric. Owns: `crates/cpl-codegen`, `cpl bench`. Worklog: [worklog.md](worklog.md).

## Covers
C++ codegen quality, runtime speed, C++ size, zero-cost abstractions, compile speed.

## Improve
Devirtualization, bounds-check elimination from proofs, narrow storage/niches; fix the over-budget programs (`ranges/mo`, `cf2178h`); iterators (12x) and slices (9x) in the zero-cost bench.

## Success criteria
Runtime p50 <= 1.1x and p95 <= 1.5x matched C++ (`cpl bench --corpus`); zero-cost bench <= 1.05x for domains, optionals, iterators, slices; every solution < 32 KB; front end < 500 ms on the largest port.

## Ryan's role
Signs off the targets once. Hears about trade-offs only (e.g. faster C++ that is harder to read).
