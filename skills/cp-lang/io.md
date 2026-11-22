# Input and Output

CP Lang reads input by expected type via `read<T>()` and writes output via
`print(...)`. No `fn main()` is needed: top-level statements execute in order.

## Reading Input

`read<T>()` parses tokens according to `T` and validates every domain, range,
and shape constraint in checked builds:

```cp
let n = read<1..=100_000>()
domain V = 1..=n
let a = read<[V]Int>()
print(a.sum())
```

Common shapes:

- Scalar: `let n = read<Int>()`, `let s = read<Str>()`, `let x = read<Float>()`
- Tuple / record: `let (n, m) = read<(Int, Int)>()`
- Domain-indexed array: `let a = read<[V]Int>()`
- Edge list: `let edges = read<[E](u: V, v: V)>()` or weighted
  `read<[E](u: V, v: V, w: Int)>()`
- 2D grid: `let g = read<[R, C]Int>()`, or `read<[R, C]Char>()` for a character grid
- Tree (reads `V.count - 1` edges): `let edges = read<Tree<V>>()`

Input contract violations (bad token or out-of-range value) trap with `R0501`
or `R0502` under `cpl run`.

## Printing Output

`print(...)` separates arguments with single spaces and appends a newline.
Sequences (`List`, `[D]T`, generators) print their elements space-separated:

```cp
let xs = [10, 20, 30]
print(xs)
print("sum:", xs.sum())
// output: 10 20 30
// output: sum: 60
```

### Booleans Require `yes(...)`

`Bool` does not implement `Print` (`E0606`). Use `yes(b)` for `YES`/`NO`, or
pass custom strings:

```cp
print(yes(true), yes(false))
print(yes(true, ("Alice", "Bob")))
// output: YES NO
// output: Alice
```

```cp error(E0606)
print(true)
```

### Optionals Require `none:` (or Unwrapping)

Print an optional directly by giving the fallback spelling for `None`, or
unwrap it with `??` / `!`:

```cp
let ans: ?Int = None
let best: ?Int = Some(42)
print(ans, none: -1)
print(best, none: -1)
// output: -1
// output: 42
```

When printing a sequence of optionals, `none:` applies to every `None` element:

```cp
domain I = 0..3
var dist: [I]?Int = None
dist[I(0)] = 0
dist[I(2)] = 5
print(dist, none: -1)
// output: 0 -1 5
```

## Multitest Problems

Put `@!multitest` at the top of the file when stdin starts with a test count
`t`. The compiler wraps the entire top-level body in a `t`-iteration loop so
per-test variables cannot leak between test cases:

```cp
@!multitest
let n = read<Int>()
print(n * 2)
```

For interactive problems, `print(...)` flushes before the next `read<T>()`,
and `flush()` is available when explicit flushing is needed.
