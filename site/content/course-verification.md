A clean solution is only half the contest loop. You also want fast binary search without off-by-one guesses, cached recursion with `@memo`, and in-file stress tests (`@brute`, `@checker`, `@interactor`) that shrink counterexamples automatically before you submit.

## Free-function binary search

CP Lang v2 uses three canonical binary search functions on any ordered range (`bsearch_first`, `bsearch_last`, `bsearch_first_some`), plus `xs.first_ge(x)` and `xs.last_le(x)` on sorted collections:

```cp CP Lang v2
let min_cap = bsearch_first(1..=1e9, cap => can_ship(cap))
let max_k = bsearch_last(0..=n, k => cost(k) <= budget)
let found = bsearch_first_some(lo..=hi, x => try_assign(x))
```

All three return an optional (`?I` or `?(I, W)`), so “no value satisfies the predicate” is `none` rather than a sentinel. On `F64` ranges, pass `steps: 80`.

## Memoization with `@memo`

Attach `@memo` to a pure recursive function over any `Index` parameters:

```cp CP Lang v2
@memo
fn ways(i: V, rem: 0..=cap) -> Int:
    ...
```

When the key domain fits in `<= 2^26` states, `@memo` allocates a flat dense table; larger keys automatically switch to a salted hash map. In the offline WebAssembly playground below, `@memo` caching is prototype-only, so the worked example runs the same recurrence directly:

```example memo
```

## Co-located `@brute`, `@checker`, and `@interactor`

Keep your slow oracle, constructive validator, or local interactive judge inside the same `.cp` file—judge builds strip all verification attributes automatically:

```cp CP Lang v2
@brute(main)
fn slow():
    ...

@checker
fn check():
    let out: [V]Int = read_output()
    assert valid(out)

@interactor
fn judge():
    let secret = read()
    while let q = next_query():
        reply(if q < secret: -1 elif q > secret: 1 else: 0)
```

In interactive solutions, call `ask("?", x, limit: 2)` so local runs enforce the query limit. Use `assume` for input preconditions (stripped on judge), `assert` for invariants that always stay checked, and `debug(...)` for side-by-side shrinking traces during `cpl stress`.

## Practice

```exercise verify-memo-output
```

```exercise verify-brute-part
```

```exercise verify-shrink
```

```exercise verify-checker
```

```exercise verify-query-budget
```
