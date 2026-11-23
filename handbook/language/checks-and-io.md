# Checks and I/O

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/09-contexts-checks.md`, `spec/core/10-io.md` |
| Describes | `spec/core/09-contexts-checks.md`, `spec/core/10-io.md`, `std/io.cp`, `spec/conformance/core-09/`, `spec/conformance/core-10/` |

How a CP Lang program talks to the judge and how it fails: the two build
modes and what each checks, contexts (the one mechanism behind `with io`,
`with checks` and `with lint`), the program's shape, reading, printing,
multitest and interaction. Terms are in `spec/GLOSSARY.md`; the whole
language is in [overview.md](overview.md).

## The problem

The first 15 lines of a C++ contest file are ceremony, and each line is a
place to be wrong:

```cpp
#define int long long            // or not, and then an overflow
ios::sync_with_stdio(false); cin.tie(nullptr);   // forget it: TLE on 10^6 tokens
int t; cin >> t; while (t--) solve();            // and clear the globals per test
cout << ans << endl;             // endl flushes: slow, except in interactive
                                 // problems, where forgetting it hangs: Idleness limit exceeded
```

Then the program itself fails in ways that are hard to explain: `n` was
read as 0 because the input was malformed and a loop went past an array;
`a[i] * a[j]` overflowed in a test with 10^9 values; `-D_GLIBCXX_DEBUG`
was the only safety net and it was off on the judge, as it should be,
because it is slow. A contestant wants three things at once: the program
fast on the judge, every mistake loud while testing, and no lines spent
on I/O.

## The model

### Two builds, one meaning

```cp
// input: "3\n4 7 1000000000\n"
// output: "1000000011"
let n: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
print(a.sum())
```

This program is the whole file. Every fact is written once:

- **In a checked build** (`cpl run`, `cpl test`, `cpl stress`, the
  interpreter, `cpl build --checked`), the read of `n` traps if the token
  is outside `1..=2e5`, each `a[i]` read traps outside `1..=1e9`, and
  `a.sum()` is proved not to overflow: at most `2e5 * 1e9`, well inside
  `Int`. A trap names the value, the bound and the source line.
- **In a release build** (`cpl build`, `cpl submit`, the bundle the
  judge compiles) none of that runs. The reads are a hand-rolled parser
  over one bulk read, and the output is one buffer written at exit.

A valid program behaves the same in the interpreter, the checked C++
and the release C++. A program that would trap in a checked build is
invalid, and in release it does whatever its C++ does: usually a wrong
answer, as the hand-written C++ would. On the judge only valid programs
matter, so release optimizes for them.

### The three questions

Every checked operation answers three questions: **what it means**,
**what the compiler can prove**, and **what each build does**. One table
in the spec (`spec/core/09-contexts-checks.md` §9.3) answers them for
every check, and every check has one name used by `with checks(...)`,
`release_checks:` and the diagnostics:

| Check | Example | Proved away when | Checked build | Release |
|---|---|---|---|---|
| `bounds` | `D(x)` from an `Int`, a `List` index | range facts show it fits | trap with value and bounds | unchecked |
| `ranges` | store into `0..7` | the value's range fits (`x % 7`) | trap naming value and range | truncates to the width |
| `overflow` | `Int` multiplication | interval arithmetic proves it fits | trap with operands | wraps |
| `unwrap` | `x!` on `None` | flow proves `Some` | trap naming the place | reads the stored value |
| `div` | `a / b`, `sqrt` of a negative | the divisor's range excludes 0 | trap | defined fallback (0) |
| `asserts` | `assert c` | never | trap with the message | skipped |
| `reads` | `read<1..=n>()` | never | trap with the input position | skipped |

Indexing `a[i]` with `i: D` on `[D]T` is not in the list: the type is
the proof, so it is never checked in either build. That is why domains
matter for speed as well as safety: the check moves to the one place a
position is made from an `Int`, and from there on it is free.

A check **never calls user code.** A checked build makes exactly the
calls to user functions, closures and input that release makes, in the
same order; a check only inspects values the program already computed.
So an interactive query, a counter in a closure or a `@cache` sees the
same history in both modes.

Beyond traps, two facts are **promised** rather than checked: `law`
declarations on operations and `@cache` correctness. Stress testing
samples them (`cpl stress` compares cold and cached runs, and tests each
law on random values). And two constructs are **unsafe**: `@trusted:`
blocks and `@cpp(...)` inline C++. `cpl explain` lists every unsafe
site.

### Contexts: settings are lexical

```cp
// output: "1,2\n1 2"
fn plain(a: Int, b: Int) = print(a, b)
with io(sep: ","):
    print(1, 2)
    plain(1, 2)
```

A **context** is a library-declared record of compile-time settings
with defaults. `with name(field: value)` sets it for the rest of the
block (a bare line, which must come first in its block) or for an
indented block. The precedence is: a call argument, then the innermost
`with`, then outer ones, then the default.

Contexts are **lexical**: `plain` above prints with a space, because its
body was written where `io.sep` is `" "`. A function that should follow
its caller says `uses io`; the caller's value is then an implicit
compile-time argument, and each distinct value gets one specialized
copy, listed by `cpl explain`. The standard contexts are `io`, `checks`,
`lint` and `stress`; a user library declares its own the same way.

### Turning checks on and off

```cp
// output: "1000000000000000000"
let big = 1000000000
with checks(overflow: on, ranges: on):
    print(big * big)
```

- `with checks(name: on)` keeps checks in release for a block or file; a
  judge profile keeps them for a whole submission with
  `release_checks: [ranges, overflow]`.
- `with checks(name: off)` removes a check in checked builds too, for a
  hot loop whose stress runs are too slow.
- `unchecked:` turns every check off in every build, and is easy to
  grep for.
- `with lint(name: allow)` silences one lint in a scope; it is the only
  in-code spelling.

### Program shape and multitest

A program is a flat sequence of top-level statements run top to bottom.
There is no `main` and no `solve`. `@!multitest` is a divider: everything
above it runs once, then the test count is read, and everything below it
runs once per test case, in a fresh scope.

```cp
// input: "3\n7\n9\n1000003\n"
// output: "YES\nNO\nYES"
let sv = Sieve(2e6)

@!multitest
let n: 2..=2e6 = read()
print(yes(sv.is_prime(n)))
```

Deleting the divider gives the single-test program. Moving setup above
it shares it across cases. State below it can't leak between cases,
because each case gets fresh `let`s, `var`s, domains and caches.
`@!multitest(until_eof)` runs until the input ends, and top-level
`return` ends the current case.

### Reading and printing

`read<T>()` reads any type from its declaration: a range type validates
the statement's bounds, `[n]T` reads `n` values, a record reads its
fields in order, `[R, C]Char` reads a grid one line per row, an enum
reads a tag and then that variant's fields. A domain written with
explicit ends (`1..=n`) reads and prints its positions as their
integers. A counted domain (`[n]T`, `domain E = [m]`) is 0-based, and
`with io(base: 1)` converts 1-based input to it at every read and back
at every print:

```cp
// input: "4\n2 3 9 1\n"
// output: "3 9"
with io(base: 1)
let n = read<1..=2e5>()
domain P = [n]
let a = read<[P]Int>()
let top = a.argmax()!
print(top, a[top])
```

`print(x, y)` joins by `io.sep` and ends the line; a sequence prints on
one line. `Bool` and `?T` are not printable on their own: `yes(b)` prints
the profile's answer words (`YES`/`NO` by default, or
`with io(answers: ("Alice", "Bob"))`), and an optional is printed with
`x ?? d` or `print(xs, none: d)`. Floats print with 10 decimals unless
`io.precision` or `precision:` says otherwise.

### Fast I/O, buffering and interaction

Input is one bulk read and a hand-rolled parser; output is one buffer
written at exit. There is nothing to write to get fast I/O, and nothing
to forget. An interactive problem constructs a `Wire`, and from then on
every read and every exit flushes first. A query is one call:

```cp
// input: "7\n"
// output: "? 1 2\n! 7"
let wire = Wire()
let x: Int = wire.ask("? 1 2")
print("!", x)
```

`wire.ask(q)` prints a line, flushes and reads the reply as the type the
call site expects. A program that never constructs a `Wire` never flushes
early, so non-interactive output stays one write. A trap in any mode
flushes buffered output before writing its report to stderr. Locally,
`cpl run` on a file with an `@interactor` fn runs the program against it
(`spec/tools/stress.md`).

## Key decisions

**Release runs no checks by default (decision 148).** On the judge a
wrong answer is a wrong answer whether it is a crash or a silent wrong
value, and every check costs time on every valid run. Lost: keeping
unproven checks in submissions, the older rule. It made safety cost
judge time on exactly the programs that were already right, and pushed
contestants to `with checks(...: off)` everywhere, which then weakened
local testing too. The opt-in (`with checks(...: on)`,
`release_checks:`) covers the contestant who wants a crash instead of a
wrong answer, for example to probe a hidden test. Reverse: default
`release_checks` to the checked-build set.

**Range stores are checked through one lowering (decision 100).** Every
store into a range-typed place lowers through one check that both the
interpreter and the checked C++ run, with identical diagnostics, and
that release compiles to a passthrough. Before it, a narrowed
`0..=1_000_000` field truncated silently in C++ while the interpreter
printed the full value: the two engines disagreed on the same program.
Proved stores (literals, a narrower source, `x % P` into `0..P`) pay
nothing in either build.

**Checks never call user code.** If a checked build called a comparator
again to validate a sort, or a closure to recheck a bound, an
interactive program would send extra queries and a `@cache` would see
different calls. The rule makes "same behaviour in both builds" true by
construction instead of by care.

**Contexts instead of flags (decisions 243, 145).** `io(base: 1)`,
`checks(overflow: on)`, `lint(float_eq: allow)` and a user library's own
settings are one mechanism with one precedence rule. Lost: an attribute
per feature, for example `@allow(name)`, which is removed syntax
(E0503) because a general mechanism already said the same thing.
Contexts are lexical rather than dynamic, because a dynamically scoped
setting makes a function's output depend on who called it, and because
a compile-time value is what lets `- io.base` fold away. Runtime values
are never contexts: a table built under one modulus must not be read
under another, so a runtime modulus is a fresh type, `type Z = Mod(p)`.

**The program is its statements (decision 20).** `@!multitest` is a
divider, not a wrapper: making a solution multitest is adding one line
and lifting shared setup above it, never re-indenting. Lost: a required
`fn solve()`, which every contestant writes and which makes "clear the
globals" a bug class. Each case being a fresh scope removes that class. Problems that
judge a function instead of a program put it in a `@grader mod`
(decision 136), so the top level stays the local driver.

**Reads come from types (chapter 10).** `read<T>()` dispatches on the
expected type, so the bound in the statement, the array length and the
record shape are the reader. Lost: `Graph.read` and other per-type
readers (R10.2.5): edges are records, so one read rule covers every
input shape, and graphs are built from the records. Input that is not
needed is still read and checked (decision 379), and statements with no
count use `read_all` (decision 161).

**Output mirrors input, and `?T` and `Bool` don't print (decision 165).**
Grids print the way they read; counts are never written for you. A
`Bool` has no one right spelling in output (`YES`, `Yes`, `1`), so it
goes through `yes(b)` and the profile's answer words; an optional has no
spelling either, so the program says what absence prints as.

**Interaction is one query call (decision 165, 382).** `wire.ask` is
print, flush, read as one call, because the blind attempts at
interactive problems all wrote it that way and the ports spent three
lines on it. Flushing is tied to the `Wire`, so it costs nothing in the
programs that don't need it. Under `@!multitest` the harness passes the
test count and runs the interactor once per test.

## Worked examples

A multitest problem with a shared table, a per-case domain and 1-based
indices: for each case, print the position of the largest value and how
many values are prime.

```cp
// input: "2\n3\n4 7 5\n2\n10 1\n"
// output: "2 2\n1 0"
let sv = Sieve(1e6)

@!multitest
let n = read<1..=2e5>()
domain I = 1..=n
let a = read<[I]1..=1e6>()
print(a.argmax()!, a.count(x => sv.is_prime(x)))
```

A sentinel-ended input with no count: read numbers until `0`.

```cp
// input: "5 3 8 0\n"
// output: "16"
let xs = read_all<Int>(until: x => x == 0)
print(xs.sum())
```

A command stream read as tagged records:

```cp
// input: "3\nadd 5\nadd 7\nsum\n"
// output: "12"
enum Cmd:
    @tag("add") Add(x: Int)
    @tag("sum") Sum
var total = 0
let q = read<1..=2e5>()
for c in read<[q]Cmd>():
    match c:
        Add(x) => total += x
        Sum => print(total)
```

## Where the precise rules are

| Topic | Rules |
|---|---|
| Contexts, precedence, `uses` | `spec/core/09-contexts-checks.md` §9.1, R09.1.1 to R09.1.5 |
| Build modes, checks never call user code | §9.2, R09.2.1 to R09.2.3 |
| The check table and check names | §9.3, R09.3.1 to R09.3.9 |
| `with checks`, `unchecked:`, `with lint` | §9.4, R09.4.1 to R09.4.5 |
| Trust tiers | §9.5, R09.5.1 to R09.5.4 |
| Recursion and the stack | §9.6, R09.6.1 |
| Program shape, `@!multitest`, graders | `spec/core/10-io.md` §10.1, R10.1.1 to R10.1.3 |
| Reading | §10.2, R10.2.1 to R10.2.9 |
| Printing | §10.3, R10.3.1 to R10.3.7 |
| Interaction and flushing | §10.4 |
| `read` and `print` signatures | §10.5 |
| Cost of I/O | §10.6 |
| Conformance cases | `spec/conformance/core-09/`, `spec/conformance/core-10/` |

## Changes

- 2026-11-23: first version.
