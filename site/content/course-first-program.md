This chapter writes, reads, and runs complete CP Lang 0.2.2 programs. By the end you will know how a program reads typed input, structures single-test and `@!multitest` files symmetrically with top-level `return`, updates state in a loop, and builds a native binary.

> **What runs.** Every `*.cp · WASM runnable` block compiles and runs in the real Rust `cpl` WebAssembly runtime on this page (and emits C++17). Blocks labeled **spec** or **prototype tested** show approved 0.2.2 forms that are still landing in the WASM frontend; see [getting started](getting-started.html#what-works-today) for full toolchain status.

## A program is the body of main

Here is a complete solution to "read `n` numbers and print their total":

```example sum
```

There is no `main`, no include list, and no fast-I/O incantation. Top-level statements run from top to bottom, like the body of `main` in C++. Top-level helper functions can read `let` bindings declared above them without threading global state through parameter lists.

Read the four lines as four decisions:

1. `let n: 1..=2e5 = read()` reads one integer and states its legal range.
2. `domain V = 1..=n` names the positions `1` through `n` as a nominal type `V`.
3. `let a: [V]Int = read()` reads one `Int` per position of `V`.
4. `print(sum(a))` adds the elements and prints the total with a newline.

```exercise first-program-predict-sum
```

## Input says what is legal

A type such as `1..=2e5` is the constraint from the problem statement, written where the value enters the program. `..=` includes its end and `..` excludes it, so `1..=2e5` and `1..200001` are the same set. `2e5` is an integer literal, not a float.

The constraint does three jobs at once:

- **Validation.** In the approved design, local builds trap if `read()` sees a value outside the range; judge builds skip the check. (Prototype builds do not enforce read bounds yet.)
- **Storage.** Range-typed array elements pack to the smallest integer width that fits while arithmetic stays 64-bit `Int`.
- **Generation.** `cpl stress` uses these declarations to generate legal random inputs; see [co-located verification](course-verification.html).

A constraint can reference an earlier binding:

```cp Prototype tested · input fragment
let n: 1..=2e5 = read()
let k: 1..=n = read()
```

Plain `Int` is 64-bit signed, so summing `2e5` values up to `1e9` never overflows. State the input range and compute in `Int`.

```exercise first-program-range-meaning
```

## let, var, and a loop

`let` binds an immutable value; `var` binds one that is reassigned. Default to `let` so every `var` visibly marks mutable loop state.

```example accumulation
```

`for v in V:` visits every position of `V` in order, and `a[v]` reads the element at `v`. A loop can update several variables at once:

```cp Prototype tested · max-and-evens.cp
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()

var best = a[V.first]
var evens = 0
for v in V:
    best = max(best, a[v])
    if a[v] % 2 == 0:
        evens += 1
print(best, evens)
```

For `5` / `-3 8 -4 7 8` this prints `8 3`. Three details carry through every chapter:

- `V.first` is the first position, so `best` starts from a real element instead of a sentinel like `-1e18`.
- Blocks use indentation, not braces; logical operators are `and`, `or`, `not`.
- `/` and `%` are Euclidean: `-3 % 2` is `1` (never negative), and `-7 / 3` is `-3`. `//` is always a line comment.

```exercise first-program-predict-accumulation
```

```exercise first-program-let-or-var
```

## Top-level return and `@!multitest`

Single-test and multi-test solutions stay structurally symmetric at top level—there is no `fn solve()` wrapper to indent your entire program into.

In a single-test program, a bare top-level `return` exits the program cleanly (returning a value like `return 0` at top level is a compile error):

```example early_return
```

```exercise first-program-early-return
```

When a problem starts with a test-case count `t`, place the file-level directive `@!multitest` on its own line (`@!` marks a file-level directive applying from that line down, while `@` always attaches to the single item below it). Here is a complete `@!multitest` program running in WebAssembly—precomputing `tri` once above the divider, exiting a zero-case early with top-level `return`, and resetting a `@memo` cache for each case's `step`:

```example multitest_divider
```

Use `@!multitest(until_eof)` when the judge gives test cases until EOF instead of a leading count. Code above `@!multitest` runs once for precomputation; code below runs per test case with fresh local bindings and emptied `@memo` caches. A second `@!multitest` divider or a nested `@!multitest` inside a block is a compile error.

```exercise first-program-multitest-output
```

```exercise first-program-multitest-divider
```

## Printing answers

`print` writes its arguments separated by single spaces and ends the line. Arrays print space-separated on one line, and `Bool` prints `YES` or `NO`:

```cp CP Lang 0.2.2 · fragment
print(best, evens)      // 8 3
print(a)                // -3 8 -4 7 8
print(best > 0)         // YES
```

## Run it natively today

Both the Rust CLI (`cpl`) and the Python prototype (`proto/cp.py`) emit readable C++17:

```sh
cpl check site/examples/sum.cp
cpl run site/examples/sum.cp < input.txt
cpl build site/examples/sum.cp -o sum.cpp && g++ -std=c++17 -O2 sum.cpp -o sum
```

When using the Python prototype on a tracked file, copy it to a scratch directory first because `proto/cp.py build` stamps a version header into the source file it reads:

```sh
mkdir -p scratch/first && cp site/examples/sum.cp scratch/first/sum.cp
python3 proto/cp.py build scratch/first/sum.cp
g++ -std=c++17 -O2 scratch/first/sum.cpp -o scratch/first/sum
printf '5\n3 1 4 1 5\n' | scratch/first/sum
```

```exercise first-program-toolchain
```

Next, [domains and boundaries](course-domains.html) shows how `domain V` prevents index mix-ups and removes `+ 1` shifts from prefix sums and windows.
