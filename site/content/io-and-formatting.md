Contest programs spend a surprising share of their code reading structured input, formatting answers cleanly, and adapting to single-test, multi-test, interactive, or IOI-grader judges. CP Lang turns all of those into declarative one-liners backed by fast buffered C++17 I/O.

## Typed `read()`

The target type drives `read()`. Reading scalars, tuples, records, fixed-size arrays, grids, and strings uses one uniform call:

```cp CP Lang 0.2.2
let n: 1..=2e5 = read()
let (l, r): (Int, Int) = read()
domain V = 1..=n
let a: [V]Int = read()
let edges: [0..m](u: V, v: V, w: Int) = read()
let grid: [Row, Col]Char = read()
```

### Dependent bounds and domain values

Range bounds can reference earlier `let` bindings (`let k: 1..=n = read()`), and reading a domain value `let u: V = read()` automatically parses the input integer and checks that it belongs to `V`. In local runs (`cpl run`, `cpl stress`), out-of-range inputs trap immediately; in `cpl build` release C++17, fast buffered parsing eliminates `cin`/`sync_with_stdio` boilerplate.

### Sentinels, EOF loops, and raw lines

When a problem uses a sentinel integer like `-1` to mean "no parent" or "missing", decode it directly into `?T` at the boundary:

```cp CP Lang 0.2.2
let parent: [V]?V = read(none: -1)      // maps -1 directly to none
```

For problems that terminate on a sentinel value or EOF, or read full lines containing spaces:

```cp CP Lang 0.2.2
for (n, m) in read_until((0, 0)):       // stops when (0, 0) is read
    ...

let line: Str = read_line()             // reads a full ASCII line (strips trailing \r/\n)
```

### Character-encoded grids and queries with `@tags`

Attach `@tags("...")` to a unit enum to map single-character tokens straight to enum variants during `read()` and `print()`:

```cp CP Lang 0.2.2
@tags(".#SE")
enum Cell: Empty; Wall; Start; End

let grid: [Row, Col]Cell = read()       // reads n rows of '.', '#', 'S', 'E' directly
```

## Output with `print()` and format strings

`print(...)` separates arguments with single spaces and writes a trailing newline. Arrays and lists print space-separated on one line, and `Bool` prints `YES` or `NO` (the standard contest convention):

```cp CP Lang 0.2.2
print(ans)                              // single value
print(a, b)                             // space-separated: "3 7"
print(arr)                              // space-separated elements on one line
print(arr, sep: "\n")                   // one element per line
print(dist, none: -1)                   // prints ?Int array, writing -1 for every none
print(ok)                               // YES or NO
```

### String interpolation and format specifiers

Interpolate any expression inside `"..."` with `{expr}` and optional format specifiers after `:`:

```cp CP Lang 0.2.2
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
let cb = Comb<1_000_000_007>(200_000)

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

## Interactive problems (`ask`)

In interactive problems, forgetting `flush` causes Idleness Limit Exceeded. Use `ask(...)` — it prints the query, flushes stdout automatically, and parses the judge's reply into the expected type:

```cp CP Lang 0.2.2
let ans = bsearch_first(1..=1_000_000_000, x => {
    let reply: Str = ask("?", x, limit: 30)
    reply == ">="
})!
print("!", ans)
```

Passing an optional `limit:` lets the co-located `@interactor` enforce your query budget locally and print the exact failing transcript if you exceed it.

## IOI / function-only graders (`@grader`)

For IOI-style problems where the judge supplies `main()` and calls your functions (while exposing grader callbacks), mark extern and entry functions with `@grader`:

```cp CP Lang 0.2.2
@grader("extern")
fn query(l: Int, r: Int) -> Int

@grader("entry")
fn solve(n: Int) -> Int:
    ...
```

When compiling with `cpl build`, CP Lang emits the exact C++ function signatures expected by the judge header instead of generating a `main()` function.
