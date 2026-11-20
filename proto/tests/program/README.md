# Program structure (decision 6.3)

A program is its top-level statements, unindented, with or without test cases. `@!multitest` on a line of
its own divides the file: what is above runs once, what is below runs once per test case (the count is read
first), each time in a fresh scope. A top-level `return` ends the program, or the current test case below
the divider.

Each `x.cp` here is built by `cpc` (run under ASan + UBSan) and run by `cpi`; both must turn `x.in` into
`x.out`. `python3 tests/test_program_structure.py` runs them, and `run_all.sh` runs that.

- `return_single` — a top-level `return` ends a single-test program, from the top level and from inside a loop.
- `divider_precompute` — a sieve above `@!multitest` is built once and shared by every test case.
- `return_multitest` — a top-level `return` below `@!multitest` ends that test case only; from nested loops
  and from a `for` over a generator too.
- `memo_reset` — a `@memo fn` below `@!multitest` starts each test case with an empty cache and fresh inputs.
- `return_above_divider` — a `return` above `@!multitest` ends the whole program.
- `divider_until_eof` — `@!multitest(until_eof)` as a divider, with shared state above it.

- `divider_legacy` — the deprecated bare `@multitest` divider still builds, for ports not yet migrated.

The matching rejections are `return_value_top`, `multitest_twice`, `directive_unknown` and `directive_nested`
in `tests/bad/`.
