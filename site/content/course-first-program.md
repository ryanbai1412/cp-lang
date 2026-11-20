This chapter writes, reads, and runs complete CP Lang programs. By the end you will know how a program reads typed input, which bindings may change, how a loop accumulates an answer, and exactly which commands produce a native executable today.

> **What runs.** Every block labeled **prototype tested** compiles with the Python prototype in `proto/` and runs as native C++17. Blocks labeled **design** describe the approved language and are not claimed to run. The Rust `cpl` CLI checks a subset of the language but cannot run or build programs yet; [getting started](getting-started.html#what-works-today) has the full status.

## A program is the body of main

Here is a complete solution to "read `n` numbers and print their total":

```example sum
```

There is no `main`, no include list, and no fast-I/O incantation. The file's top-level statements run from top to bottom, exactly like the body of `main` in a C++ solution. Functions you declare at the top level can see the `let` bindings above them, so helpers do not need long parameter lists of global state.

Read the four lines as four decisions:

1. `let n: 1..=2e5 = read()` reads one integer and states its legal range.
2. `domain V = 1..=n` names the positions `1` through `n` as a new type, `V`.
3. `let a: [V]Int = read()` reads one `Int` for every position in `V`.
4. `print(sum(a))` adds the elements and prints the total with a newline.

The next chapter is entirely about the second line. For now, read `[V]Int` as "an array with one `Int` per position of `V`", and notice that the reader knows how many numbers to consume because the array's index type says so.

```exercise first-program-predict-sum
```

## Input says what is legal

A type such as `1..=2e5` is not just documentation. It is the constraint from the statement, written where the value enters the program. `..=` includes its end and `..` excludes it, so `1..=2e5` and `1..200001` are the same set. `2e5` is an integer literal here, not a float.

The constraint does three jobs at once:

- **Validation.** In the approved design, a local build that reads `0` for `n` stops at the read instead of producing a confusing wrong answer later; the judge build skips the check. The prototype does not validate reads yet, so treat this as the target behavior.
- **Storage.** A range-typed array element uses the smallest width that fits, while arithmetic still happens in 64-bit `Int`.
- **Generation.** The stress tester uses the same declarations to produce legal random inputs; see [memoization & stress tests](testing.html#turn-input-into-a-specification).

A constraint can mention an earlier value. This is the common "`1 <= k <= n`" line from a statement, and it runs in the prototype:

```cp Prototype tested · input fragment
let n: 1..=2e5 = read()
let k: 1..=n = read()
```

Plain `Int` is the default 64-bit signed integer, so a sum of `2e5` values up to `1e9` does not overflow. The habit to drop is choosing between `int` and `long long` by guesswork: state the input range, and compute in `Int`.

```exercise first-program-range-meaning
```

## let, var, and a loop

`let` binds a value that is never reassigned. `var` binds one that is. Use `let` by default; a `var` then marks exactly the state that the loop below it updates.

```example accumulation
```

`for v in V:` visits every position of `V` in order, and `a[v]` is the element at that position. `total` starts at `0`, which the compiler reads as an `Int`, and `+=` updates it. Notice the element type: `[V]0..=1000` promises every element is between `0` and `1000`. That promise sizes the storage and bounds the generator; rejecting an input such as `2000` at the read is the approved local-build behavior, not yet a prototype check.

A loop can update several pieces of state at once. This program prints the largest value and how many values are even:

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

For input `5` / `-3 8 -4 7 8` it prints `8 3`. Three details carry over to every later chapter:

- `V.first` is the first position, so `best` starts from a real element rather than from a sentinel such as `-1e18`.
- Blocks are indentation; there are no braces. Logical operators are written `and`, `or`, and `not`.
- `%` is Euclidean: the remainder is never negative. In C++, `-3 % 2` is `-1`, and an "is odd" test written as `x % 2 == 1` silently misses negative numbers. Here `-3 % 2` is `1`, so either test is correct.

```exercise first-program-predict-accumulation
```

```exercise first-program-let-or-var
```

## Printing answers

`print` writes its arguments separated by single spaces and ends the line. An array prints as its elements on one line, and a `Bool` prints as `YES` or `NO`, the spelling most judges ask for.

```cp Prototype tested · fragment
print(best, evens)      // 8 3
print(a)                // -3 8 -4 7 8
print(best > 0)         // YES
```

These lines assume the bindings of the previous program. When a problem asks for one answer per line, call `print` once per answer. `//` starts a comment; it is never integer division. Division is `/`, and like `%` it is Euclidean, so `-7 / 3` is `-3`. The [language tour](tour.html#a-few-c-habits-to-leave-behind) lists the other C++ habits to leave behind.

## Run it natively today

The prototype translates a `.cp` file into one readable C++17 file, which you compile and run with your own compiler. Work in a scratch directory, because the build adds a compiler stamp line to the top of the source file it reads and writes the `.cpp` beside it:

```sh
mkdir -p scratch/first && cp site/examples/sum.cp scratch/first/sum.cp
python3 proto/cp.py build scratch/first/sum.cp
g++ -std=c++17 -O2 scratch/first/sum.cpp -o scratch/first/sum
printf '5\n3 1 4 1 5\n' | scratch/first/sum
```

The last command prints `14`. The generated `sum.cpp` is the file you would submit to a judge; the `.cp` file is your source. Never build a tracked file such as a corpus port in place: the stamp would show up as a change in Git.

The Rust CLI is real but younger. `cpl check sum.cp` runs the shared frontend over the subset it understands, and `cpl lsp` serves editor features. `cpl run` and `cpl build` check the file and then report that their backends are not built; they do not execute it. The planned contest loop of `cpl run`, `cpl test`, `cpl stress`, and `cpl build` is described in [getting started](getting-started.html#the-intended-contest-loop) and is not something to rely on yet.

```exercise first-program-toolchain
```

## Common mistakes

**Reading without a range.** `let n: Int = read()` compiles, but the stress tester then has no idea what a legal `n` is, and once local input validation lands it has nothing to check. Copy the statement's constraint into the type.

**Starting a maximum at a sentinel.** `var best = -1e18` is a guess that a later subtask can break. Start from `a[V.first]`, or reduce with `max(...)`, which returns an optional that is `none` for an empty input.

**Expecting the Rust CLI to run code.** Until its backends land, the prototype commands above are the honest way to get a binary. A clean `cpl check` does not mean every feature in the file is implemented.

Next, [domains and boundaries](course-domains.html) explains what `domain V` buys you: indices that cannot be confused with each other, and prefix sums with no `+ 1`.
