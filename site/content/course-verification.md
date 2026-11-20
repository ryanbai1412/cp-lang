A fast algorithm is only half the contest loop. Before submitting, you want a slow brute-force oracle, a validator for constructive problems, or a local interactive judge—all living right beside your solution so they share the same input types and disappear automatically from the submitted C++17.

## Co-located `@brute`

Attach `@brute(main)` (or `@brute(fn_name)` to target one helper) directly inside your `.cp` file. The brute sees the same top-level `let` inputs in both single-test and `@!multitest` files, and `cpl build` strips `@brute` completely from judge submissions:

```example colocated_brute
```

Run the fast prefix-minimum solution above in the sandbox, or click **C++17** to verify that `@brute(main)` is stripped from the generated submission code.

```exercise verify-brute-output
```

```exercise verify-brute-part
```

## Co-located `@checker` for constructive answers

When a problem accepts any valid coloring, permutation, or construction, character-by-character comparison against a brute fails. Add a `@checker` block in the same file:

```cp CP Lang 0.2.2 · spec (cpl stress harness)
@checker
fn check():
    let out: [V]Color = read_output()
    assert valid(out)
```

When paired with `@brute(main)`, the brute settles **existence** (e.g., whether a valid coloring exists or the answer is `-1`), while `@checker` validates any constructive answer your fast solution prints.

> **WASM status.** `@brute(main)` and `@interactor` parse, check, and strip cleanly in WASM `run`/`build` today (`colocated_brute.cp` above). The `read_output()` builtin and the automated `cpl stress` shrinking/comparison runner are part of the approved 0.2.2 verification spec and are not executed inside the browser sandbox.

```exercise verify-shrink
```

```exercise verify-checker
```

## Co-located `@interactor` and query budgets

Interactive problems pair `ask("?", x, limit: K)` in your solution with a co-located `@interactor` judge in the same file:

```cp CP Lang 0.2.2 · spec (cpl stress interactive harness)
@interactor
fn judge(sol: Interactor):
    let secret: 1..=1000 = read()      // hidden test case
    while true:
        let q: Int = sol.recv()        // reads solution's stdout
        sol.send(if q < secret: -1 elif q > secret: 1 else: 0)
        if q == secret: return
```

Any program that calls `ask(...)` automatically flushes every `print(...)`, so forgetting `endl`/`flush` cannot cause an idleness-limit verdict. Locally, `@interactor` enforces `limit: K` and prints the full query transcript if you exceed the budget.

## Free-function binary search

For answer binary search and interactive halving, CP Lang 0.2.2 provides two canonical functions on any ordered range (`bsearch_first`, `bsearch_last`) returning `?I`:

```cp CP Lang 0.2.2 · spec fragment
let min_cap = bsearch_first(1..=1e9, cap => can_ship(cap))
let max_k = bsearch_last(0..=n, k => cost(k) <= budget)
```

Returning `?I` means "no value satisfies the predicate" is `none` rather than a sentinel. On `F64` ranges, pass `steps: 80`. For cached recurrences with `@memo`, see [Chapter 5](course-dp.html).

```exercise verify-query-budget
```

## Recursive baseline reference

Here is a compact linear recurrence (`f(0) = 1`, `f(i) = f(i - 1) * 3 + i`) for tracing small base-case boundaries:

```example memo
```

```exercise verify-memo-output
```
