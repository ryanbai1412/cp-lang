# Debugging, Testing, and Stress Testing

Checked builds (`cpl run`, `cpl test`, `cpl stress`, `cpl build --checked`)
check array/domain bounds, integer overflow, range contracts, and `None`
unwraps. Release builds (`cpl build`, `cpl submit`) drop those checks for
judge speed.

## Fast Local Cycle

```bash
cpl check sol.cp                 # type/domain/borrow check (< 500 ms)
cpl run sol.cp < sample.in       # run checked on sample input
cpl test sol.cp                  # run all @test functions in sol.cp
cpl build sol.cp                 # release binary
cpl submit sol.cp -o sub.cpp     # single-file C++17 for judge upload
```

## Inline Unit Tests (`@test`)

Write `@test` functions directly in your solution or helper file and run them
with `cpl test`:

```cp
fn tri(n: Int) -> Int = n * (n + 1) / 2

@test
fn tri_small():
    assert tri(0) == 0
    assert tri(4) == 10

print(tri(4))
// output: 10
```

## Assertions and Debug Logging

- `assert cond` traps with `R0401` in checked builds when `cond` is `false`.
- `debug(x, y)` prints values to stderr in checked builds and is erased in
  release builds (`cpl build` / `cpl submit`), so you never leak debug output
  on the judge.

## Stress Testing (`cpl stress`)

When a solution fails on hidden tests, write a small brute-force `brute.cp`
and a random test generator `gen.cp` (using `std.rand.Rng`), then run:

```bash
cpl stress sol.cp brute.cp gen.cp --runs 500
```

`cpl stress` runs both solutions on generated inputs, stops on the first
mismatch or trap, and saves the failing input so you can reproduce it directly
with `cpl run sol.cp < fail.in`.

A minimal generator using `std.rand.Rng`:

```cp
use std.rand.{Rng}

var rng = Rng(1)
let n = rng.int(1, 5)
var xs: List<Int> = []
for _ in 0..n:
    xs.push(rng.int(-10, 10))
assert xs.len == n
```

## `unchecked:` Escape Hatch

`unchecked:` is a statement block (not an expression; any trailing value is
dropped) that disables checked-build runtime traps inside a hot inner loop
when you need checked-mode speed while debugging elsewhere:

```cp
var s = 0
unchecked:
    for i in 0..10:
        s += i
print(s)
// output: 45
```
