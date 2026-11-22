# Debugging, Testing, and Stress Testing

Checked builds (`cpl run`, `cpl test`, `cpl stress`, `cpl build --checked`)
check array/domain bounds, integer overflow, range contracts, and `None`
unwraps. Release builds (`cpl build`, `cpl submit`) drop those checks for
judge speed.

## Fast Local Cycle

```bash
cpl check sol.cp                 # type/domain/borrow check (< 500 ms)
cpl run sol.cp < sample.in       # run checked on sample input
cpl test sol                     # sol.tests/ cases, @sample and @test fns
cpl stress sol.cp                # solution vs its @alt brutes, shrinks failures
cpl build sol.cp -o sub.cpp      # the one C++17 file to submit
cpl submit sol.cp                # build for its judge, then run sol.tests/ on that C++
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

Put the brute in the same file as an `@alt`. `@alt(program)` is a whole-program
brute; `@alt(f)` is a slow twin of fn `f`, compared at each call site.
`cpl stress sol.cp` generates inputs from the program's `read` types, runs
both, stops at the first disagreement, shrinks it, and saves it to
`sol.tests/stress-<seed>.in` as a regression case.

```cp
let n = read<1..=8>()
domain V = 1..=n
let a = read<[V]-5..=5>()
print(V.all.map(v => a[v]).max()!)

@alt(program)
fn brute():
    let n = read<1..=8>()
    let a = read<[n]-5..=5>()
    var best = a[0]
    for x in a: best max= x
    print(best)
// input: 3
// input: 1 -2 3
// output: 3
```

Options: `--cases N` (default 1000), `--size N` (largest n, default 8),
`--seed N`, `--trace` (saves both runs' traces on failure), `--brute PROG`,
`--gen PROG`, `--checker PROG` for external programs, and `--interactor PROG`
for interactive problems: `cpl` runs `PROG input` with the solution wired to
the interactor's stdin/stdout (the interactor reads the test from the file
named by its first argument, never spawns the solution) and its exit code is
the verdict.

Tight constraint types make better random inputs: `read<[V]-5..=5>()` already
tells stress what to generate. (`@gen fn` with `gen(...)` for constraints
types can't express is specified but not implemented yet.)

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
