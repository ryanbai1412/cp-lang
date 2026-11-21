Contest programs spend a surprising share of their code reading structured input, formatting answers cleanly, and adapting to single-test, multi-test, interactive, or IOI-grader judges. CP Lang turns all of those into declarative one-liners backed by fast buffered C++17 I/O.

## Typed `read()`

The target type drives `read()`. Always put the type on the binding (`let k: Int = read()`) — never pass a type name like `read(Int)` (only a range literal `read(1..=2e5)` or `read(none: -1)` is accepted as an argument). `read()` can fill a fixed-size array (`let a: [0..n]Int = read()`, `let a: [V]Int = read()`), **never** a dynamic `List<Int>` (whose length is unknown at compile time). Reading scalars, tuples, records, fixed-size arrays, grids, and strings uses one uniform call:

```cp CP Lang
let n = read(1..=2e5)           // ranged read shorthand (checked in run & C++)
let m: 0..=2e5 = read()
let (l, r): (Int, Int) = read()
domain V = 1..=n
domain Row = 1..=n
domain Col = 1..=n
let a: [V]Int = read()
let edges: [0..m](u: V, v: V, w: Int) = read()
let grid: [Row, Col]Char = read()
```

### Dependent bounds and domain values

Range bounds can reference earlier `let` bindings (`let k: 1..=n = read()`), and reading a domain value `let u: V = read()` automatically parses the input integer and checks that it belongs to `V`. In local runs (`cpl run`, `cpl stress`), out-of-range inputs trap immediately; in `cpl build` release C++17, fast buffered parsing eliminates `cin`/`sync_with_stdio` boilerplate.

### Sentinels and EOF loops

When a problem uses a sentinel integer like `-1` or `0` to mean "no parent" or "missing", decode it into `?T` directly at the boundary with `read(none: sentinel)`:

```cp CP Lang
let n = read(1..=2e5)
domain V = 1..=n
let parent: [V]?V = [V] _ => read(none: -1)
print(parent, none: "-1")
```

For problems that terminate on a sentinel value, write an ordinary `loop:`:

```cp CP Lang
loop:
    let (n, m): (Int, Int) = read()
    if (n, m) == (0, 0): break          // stops when (0, 0) is read
    print(n + m)
```

Use `@!multitest(until_eof)` for repeated test cases ending at EOF. `read()` currently reads whitespace-delimited tokens; a full-line `read_line()` API is not implemented.

### Character-encoded grids and state enums

Read a character grid and map its alphabet to enum states with `match`. Automatic `@tags` I/O is specification-only in the current execution backend:

```cp CP Lang 0.2.2
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 1..=n
domain Col = 1..=m
enum Cell:
    Empty
    Wall
    Start
    End

let grid: [Row, Col]Char = read()
for r in Row:
    for c in Col:
        let ch = grid[r, c]
        assert ch in ".#SE"
        let cell = match ch:
            '.' => Empty
            '#' => Wall
            'S' => Start
            _ => End
        if cell == Start: print(r, c)
```

## Output with `print()` and format strings

`print(...)` separates arguments with single spaces and writes a trailing newline. Arrays and lists print space-separated on one line, and `Bool` prints `YES` or `NO` (the standard contest convention):

```cp CP Lang 0.2.2
let ans = 42
let (a, b) = (3, 7)
let arr = [3, 7]
let dist: List<?Int> = [0, none, 7]
let ok = true
print(ans)                              // single value
print(a, b)                             // space-separated: "3 7"
print(arr)                              // space-separated elements on one line
print(arr, sep: "\n")                   // one element per line
print(dist, none: "-1")                 // prints ?Int array, writing -1 for every none
print(ok)                               // YES or NO
```

### String interpolation and format specifiers

Interpolate any expression inside `"..."` with `{expr}` and optional format specifiers after `:`:

```cp CP Lang 0.2.2
let (tc, ans, n) = (1, 42, 42)
let x = 7.0 / 3.0
let (num, den) = (1, 7)
print("Case #{tc}: {ans}")              // string interpolation
print("{n:5}")                          // width 5, space-padded
print("{n:05}")                         // width 5, zero-padded ("00042")
print("{x:.6f}")                        // fixed-point float, 6 decimal places (never -0.00)
print("{x:8.2f}")                       // width 8, 2 decimal places
print("{n:x}")                          // lowercase hex
print("{n:b}")                          // binary
print(fixed(num, den, 6))               // exact integer rational formatted to 6 decimal places
```

Notice `fixed(num, den, k)`: when a problem asks for `p / q` to `k` decimal places and you want exact rounding without `F64` precision loss, `fixed(num, den, k)` formats the exact decimal digits directly from integers.

## Single-test vs `@!multitest`

Single-test and multi-test solutions have the exact same top-level structure — there is no `fn solve()` wrapper:

```cp CP Lang 0.2.2
// Runs ONCE before the test loop (great for sieves, factorials, lookup tables):
let cb = Comb<Mod<1_000_000_007>>(200_000)

@!multitest                             // reads t and loops everything below t times

let (n, k): (1..=2e5, 0..=2e5) = read()
if k > n:
    print(0)
    return                              // ends ONLY the current test case

print(cb.choose(n, k))
```

Three rules make `@!multitest` effortless:

- `@!` is a file-level directive applying from that line downward, whereas `@` always attaches to the single declaration below it.
- Everything below `@!multitest` runs in a fresh scope for each test case, and every `@memo` function declared below it automatically resets its cache between cases.
- Bare top-level `return` exits the program in a single-test file or ends the current test case below `@!multitest`.
- Use `@!multitest(until_eof)` for judges that repeat test cases until end-of-file without a leading count `t`.

## Interactive problems (`@!interactive` and `ask`)

In interactive problems, forgetting `flush` causes Idleness Limit Exceeded. Put `@!interactive` at the top of the file to flush every `print(...)` automatically, or use `ask(...)` — it prints the query, flushes stdout automatically, and parses the judge's reply into the expected type:

```cp CP Lang 0.2.2
fn at_least(x: Int) -> Bool:
    let reply: Str = ask("?", x, limit: 30)
    reply == ">="
let ans = bsearch_first(1..=1_000_000_000, at_least)!
print("!", ans)
```

Passing an optional `limit:` lets the co-located `@interactor` enforce your query budget locally and print the exact failing transcript if you exceed it.

## IOI / function-only graders (`@grader`)

For IOI-style problems where the judge supplies `main()` and calls your functions (while exposing grader callbacks), mark extern and entry functions with `@grader`:

```cp CP Lang 0.2.2
@grader
fn query(l: Int, r: Int) -> Int

@grader
fn solve(n: Int) -> Int:
    query(0, n - 1)
```

When compiling with `cpl build`, CP Lang emits the exact C++ function signatures expected by the judge header instead of generating a `main()` function.
