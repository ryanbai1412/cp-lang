A fast algorithm is only half the contest loop. Before submitting, you want a slow brute-force oracle, a validator for constructive problems, or a local interactive judge—all living right beside your solution so they share the same input types and disappear automatically from the submitted C++17.

## Co-located `@brute`

Attach `@brute(main)` (or `@brute(fn_name)` to target one helper) directly inside your `.cp` file. The brute sees the same top-level `let` inputs in both single-test and `@!multitest` files, and `cpl build` omits `@brute` from the executable C++17 body (the original `.cp` source remains in the default comment header):

```example colocated_brute
```

Run the fast prefix-minimum solution above in the sandbox, inspect **C++17** to see that `slow()` is omitted from the executable C++ body (while the original `.cp` source remains in the header comment), or open it in the [playground](playground.html) and press **Trace** to step through the solution and `@brute(main)` side by side in the [trace debugger](trace.html).

```exercise verify-brute-output
```

```exercise verify-brute-part
```

## Co-located `@checker` for constructive answers

When a problem accepts any valid coloring, permutation, or construction, character-by-character comparison against a brute fails. Add a `@checker` block in the same file:

```cp CP Lang 0.2.2 · local cpl stress
domain V = 1..=5
print([V] v => v.to_idx % 2)

@checker
fn check():
    let out: [V]0..=1 = read_output()
    for v in V if v != V.last:
        assert out[v] != out[(v + 1)!]
```

When paired with `@brute(main)`, the brute settles **existence** (e.g., whether a valid coloring exists or the answer is `-1`), while `@checker` validates any constructive answer your fast solution prints.

> **Browser vs. local CLI.** `@brute(main)`, `@checker` (including `read_output()`), and `@interactor` parse, check, and omit verification code from the executable C++17 body in both `cpl` and WebAssembly. In the [playground](playground.html), **Trace** runs `@brute(main)` beside your fast solution on the current input and opens the live diff in the [trace debugger](trace.html). The automated `cpl stress` random generator, shrinker, and `@checker`/`@interactor` harnesses run locally via `cpl stress` (with `--trace` for side-by-side HTML captures, and `--explain` to name the first value your solution and its `@brute(main)` give a variable differently, with the line on each side), not inside the browser sandbox.

```exercise verify-shrink
```

```exercise verify-checker
```

## Co-located `@interactor` and query budgets

Interactive problems pair `ask("?", x, limit: K)` in your solution with a co-located `@interactor` judge in the same file:

```cp CP Lang 0.2.2 · local cpl stress
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

```cp CP Lang 0.2.2
let weights = [3, 2, 5, 1]
let days = 2
fn can_ship(cap: Int) -> Bool:
    var used = 1
    var load = 0
    for w in weights:
        if w > cap: return false
        if load + w > cap:
            used += 1
            load = 0
        load += w
    used <= days

let n = 10
let budget = 25
fn cost(k: Int) -> Int: k * k
let min_cap = bsearch_first(1..=1e9, cap => can_ship(cap))
let max_k = bsearch_last(0..=n, k => cost(k) <= budget)
print(min_cap, max_k)
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

## Write a program

```exercise verify-write-nonempty
```
