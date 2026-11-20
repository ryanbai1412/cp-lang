A clean solution is only half the contest loop. You also want fast binary search without off-by-one guesses, cached recursion with `@memo`, and in-file stress tests (`@brute`, `@checker`, `@interactor`) that shrink counterexamples automatically before you submit.

## Free-function binary search

CP Lang v2.2 uses two canonical binary search functions on any ordered range (`bsearch_first`, `bsearch_last`), plus `xs.first_ge(x)` and `xs.last_le(x)` on sorted collections:

```cp CP Lang v2.2
let min_cap = bsearch_first(1..=1e9, cap => can_ship(cap))
let max_k = bsearch_last(0..=n, k => cost(k) <= budget)
```

Both return `?I`, so “no value satisfies the predicate” is `none` rather than a sentinel. On `F64` ranges, pass `steps: 80`. The sorted lookups return what the container holds: a sequence gives the index (`?Int` or `?D`), a `Set` or `SortedBag` gives the key (`?K`), and a `Map` gives the pair (`?(K, V)`).

## Memoization with `@memo`

Attach `@memo` to a pure recursive function over any `Index` parameters:

```cp CP Lang v2.2
@memo
fn ways(i: V, rem: 0..=cap) -> Int:
    ...
```

When every parameter is an `Index` and the key space fits in `2^24` states (`2^26` for `Bool` results), `@memo` allocates a flat dense table; larger keys automatically switch to a salted hash map. There are no `@memo(...)` parameters, and a `@memo fn` nested in a loop resets on every iteration. In the offline WebAssembly playground below, `@memo` caching is prototype-only, so the worked example runs the same recurrence directly:

```example memo
```

## Co-located `@brute`, `@checker`, and `@interactor`

Keep your slow oracle, constructive validator, or local interactive judge inside the same `.cp` file—judge builds strip all verification attributes automatically:

```cp CP Lang v2.2
@brute(main)
fn slow():
    ...

@checker
fn check():
    let out: [V]Int = read_output()
    assert valid(out)

@interactor
fn judge(sol: Interactor):
    let secret: 1..=1000 = read()      // the hidden test
    while true:
        let q: Int = sol.recv()         // the solution's stdout
        sol.send(if q < secret: -1 elif q > secret: 1 else: 0)
        if q == secret: return
```

In interactive solutions, call `ask("?", x, limit: 2)` so local runs enforce the query limit. Any program that calls `ask(...)` flushes every `print(...)` automatically. State input preconditions as typed `read()` bounds, use `assert` for invariants (it is always checked, locally and on the judge), and `debug(...)` for side-by-side shrinking traces during `cpl stress`.

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
