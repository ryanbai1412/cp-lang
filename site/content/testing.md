Contest bugs rarely come from the idea; they come from an edge case you did not construct by hand. CP Lang puts memoization, input generators, brute-force oracles, constructive checkers, and local interactors inside the same `.cp` file, then strips verification code automatically in judge builds.

## Memoization with `@memo`

Annotate a pure recursive function with `@memo` to cache its results over any `Index` parameter tuple:

```cp CP Lang v2.2
@memo
fn dp(u: V, used: Bool) -> Int:
    ...
```

`@memo` takes no parameters. When every parameter is an algebraic `Index` and the product of their sizes is at most `2^24` (`2^26` for `Bool` results), it emits a flat niche-packed array with zero hash overhead; otherwise it falls back to a per-process salted open-addressed hash table. A `@memo fn` may be top-level or nested in a function or loop, where its table resets on every call or iteration, and it may wrap `ask(...)` or an extern `@grader fn` so each argument tuple costs one query. If you want explicit hash storage, write `var dp = HashMap<Key, Val>()`.

In the browser playground and current Rust/WASM interpreter, `@memo` caching is prototype-only, so the runnable example below executes the same recurrence with plain recursion:

```example memo
```

For input `4`, `f(3)` evaluates to `45`.

## Assertions and `debug`

There is one assertion keyword, so a sanity check can never be stripped by accident:

- `assert cond` — always checked in both local and judge builds, and it feeds the range and disjointness proofs. Input constraints are not assertions: they live in typed `read()` bounds, `read(gen: f)`, `@gen`, and `@checker`.
- `debug(x, dp)` — pretty-prints labeled values to `stderr` locally and replays side-by-side when `cpl stress` finds a failing case; stripped on judge builds.

## Co-located `@brute`, `@checker`, `@interactor`, and `@gen`

Write your slow reference, constructive validator, or interactive judge right next to your solution:

```cp CP Lang v2.2
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
fn judge(sol: Interactor):
    let (n, secret): (1..=1000, 1..=n) = read()   // the hidden test
    sol.send(n)                                    // to the solution's stdin
    var qs = 0
    while true:
        let kind: Str = sol.recv()                 // from the solution's stdout
        if kind == "!":
            let guess: Int = sol.recv()
            assert guess == secret
            return
        qs += 1
        assert qs <= 12
        let x: Int = sol.recv()
        sol.send(if x < secret: "<" else: ">=")
```

In interactive problems, pair `@interactor` with `ask("?", x, limit: 2)` so query limits are enforced locally. A program that calls `ask(...)` anywhere flushes every `print(...)` automatically. For IOI-style graders, a bodyless `@grader fn` declares an extern oracle, and a stateful grader keeps its state in a top-level `var` between `init(...)` and `query(...)`. Every generator draws from a replayable choice stream, which lets `cpl stress` shrink any failing test automatically without writing a custom shrinker.

## Running `cpl stress`

`cpl stress solution.cp` defaults to the in-file `@brute`, `@checker`, or `@interactor`, escalates test sizes (`1..=20`), compiles with `-DLOCAL`, shrinks the first failure via choice streams, replays `debug(...)` outputs side-by-side, and reports max size reached, rejection rate (warning above 50%), diff mode (`--close 1e-9` for floats), and output diversity.

For a visual side-by-side comparison of an optimized DP run against its reference, open the [trace explorer](trace.html) and inspect the 0/1 knapsack capture.

## How the site stays honest

Every worked example embedded in these pages is checked by the test suite against both the compiler and the offline WebAssembly runtime. Code blocks marked **CP Lang v2.2** show the locked specification; blocks marked **prototype tested** are executed on every build.
