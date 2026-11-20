Contest bugs rarely come from the idea; they come from an edge case you did not construct by hand. CP Lang puts memoization, input generators, brute-force oracles, constructive checkers, and local interactors inside the same `.cp` file, then strips verification code automatically in judge builds.

## Memoization with `@memo`

Annotate a pure recursive function with `@memo` to cache its results over any `Index` parameter tuple:

```cp CP Lang v2
@memo
fn dp(u: V, used: Bool) -> Int:
    ...
```

When the parameter product is an algebraic `Index` with size `<= 2^26`, `@memo` emits a flat dense array (zero hash overhead); otherwise it uses a per-process salted open-addressed hash table. Override explicitly with `@memo(flat)` or `@memo(hash)`.

In the browser playground and current Rust/WASM interpreter, `@memo` caching is prototype-only, so the runnable example below executes the same recurrence with plain recursion:

```example memo
```

For input `4`, `f(3)` evaluates to `45`.

## Assertions, assumptions, and `debug`

CP Lang distinguishes three diagnostic tools so local checks stay strict without slowing down judge submissions:

- `assume cond` — documents an input precondition; checked in local debug builds and used to filter/validate generated stress inputs, then stripped on judge builds.
- `assert cond` — always checked in both local and judge builds.
- `debug(x, dp)` — pretty-prints labeled values to `stderr` locally and replays side-by-side when `cpl stress` finds a failing case; stripped on judge builds.

## Co-located `@brute`, `@checker`, `@interactor`, and `@gen`

Write your slow reference, constructive validator, or interactive judge right next to your solution:

```cp CP Lang v2
@gen
fn make_case(size: Int):
    let n = rand(1..=size)
    emit(n)
    emit([0..n] _ => rand(-10..=10))

@brute(main)
fn slow():
    ...

@checker
fn check():
    let ans: [V]Int = read_output()
    assert is_valid(ans)

@interactor
fn judge():
    let secret = read()
    while let q = next_query():
        reply(compare(q, secret))
```

In interactive problems, pair `@interactor` with `ask("?", x, limit: 2)` so query limits are enforced locally. Every generator draws from a replayable choice stream, which lets `cpl stress` shrink any failing test automatically without writing a custom shrinker.

## Running `cpl stress`

`cpl stress solution.cp` defaults to the in-file `@brute`, `@checker`, or `@interactor`, escalates test sizes (`1..=20`), compiles with `-DLOCAL`, shrinks the first failure via choice streams, replays `debug(...)` outputs side-by-side, and reports max size reached, rejection rate (warning above 50%), diff mode (`--close 1e-9` for floats), and output diversity.

For a visual side-by-side comparison of an optimized DP run against its reference, open the [trace explorer](trace.html) and inspect the 0/1 knapsack capture.

## How the site stays honest

Every worked example embedded in these pages is checked by the test suite against both the compiler and the offline WebAssembly runtime. Code blocks marked **CP Lang v2** show the locked specification; blocks marked **prototype tested** are executed on every build.
