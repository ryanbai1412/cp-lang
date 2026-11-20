# 1.6 Program structure, modules and libraries

Status: decided unless a section says *proposed*. Folded from `program-structure.md`
(now in `archive/`); this copy is the source of truth.


How a solution is laid out, where state lives, how a solution is split
into parts you can test alone, and how multitest, brutes, checkers,
interactors, local-only code, samples and libraries fit around it.

**The guiding rule.** The core language has only general features: files,
top-level code, functions, nested functions, `const`,
decorators, `cfg` and `use`. Nothing in the core knows about contests.
Multitest, brutes, samples, checkers and interactors are stdlib decorators
or `cp` commands built from those features. Someone writing a non-contest
program never sees them.

**Status.** Built in the prototype: top-level entry, nested fns,
`@multitest`, whole-program `@brute(main)`, `@solution(when: ...)`, typed
`read()` with `where`, and `cfg.debug`. Everything else here is proposed.

---------------------------------------------------------------------------

## 1. A whole program

Here's a complete solution, to show the shape before the rules. The problem:
an array, and queries asking for the number of distinct values in a range.

```
// cp 0.4.2                                     <- compiler stamp, added automatically
use std.fenwick

let n: 1..=2e5 = read()
domain I = 1..=n
let a: [I]0..=1e9 = read()
let q: 1..=2e5 = read()
domain Q = 1..=q
let qs: [Q](l: I, r: I) = read() where l <= r

fn last_seen(a: [I]Int) -> [I]?I:
    var seen: Map<Int, I> = {}
    [I] i => { let p = seen.get(a[i]); seen[a[i]] = i; p }

fn answer(prev: [I]?I) -> [Q]Int:
    var fw = Fenwick<I, Int>(Sum<Int>)
    var ans: [Q]Int = 0
    var r = I.first
    for k in Q.sorted(by: k => qs[k].r):
        ...
    ans

let prev = last_seen(a)
print(answer(prev), sep: "\n")

@brute(answer)
fn answer_slow(prev: [I]?I) -> [Q]Int =
    [Q] k => count(i for i in qs[k].l..=qs[k].r if prev[i].map(p => p < qs[k].l) ?? true)

@sample("5\n1 2 1 3 2\n2\n1 3\n2 5\n", "2\n3\n")
```

Reading it top to bottom:

- The input is read with types, so `n`, `a` and `qs` are both read and
  validated, and the same lines describe how `cp stress` generates random
  inputs.
- The solution is two parts, `last_seen` and `answer`. Each is a
  plain function, and each can be tested, traced and replayed alone.
- `answer_slow` is a brute for one part, not for the whole program. It
  checks `answer` given `prev`, by counting first occurrences directly.
  When a stress test fails, the tools can say "`answer` agrees with its
  brute, so the bug is in `last_seen`" (or the other way round).
- The sample lives next to the code, and none of the testing code reaches
  the submission.

---------------------------------------------------------------------------

## 2. Files: programs and libraries

### 2.1 A program file is a function body

A file with top-level statements is a **program**. Its top-level
statements are the body of an implicit `main`, run top to bottom; there's
no `fn main()` to write. Its top-level declarations (`fn`, `struct`,
`enum`, `domain`, `const`) are nested inside that body, so a top-level fn
sees the body's `let`s, the same rule as any nested fn:

```
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()

fn answer() -> ?Int:              // sees n, V and a with no arguments
    ...

print(answer() ?? -1)
```

Declarations can go anywhere in the file, so you can put the main flow
first and the helpers below it. The one ordering rule is about *calls*:
calling a fn before the `let`s it reads are set is a compile error.

```
print(answer() ?? -1)             // error: answer reads `a` (line 3), which isn't set yet here
let a: [V]Int = read()
```

The compiler already knows which `let`s each fn reads (it needs that for
the tools, §4.2), so the check is cheap and the message names the `let`.

### 2.2 `return` at the top level

Since the top level is a function body, `return` works there. It ends the
program (after flushing output), or the current test under `@multitest`
(§6):

```
if n == 1:
    print(0)
    return
```

### 2.3 `static let`

A `static let` is initialized the first time it's reached, and then kept
for the rest of the run:

```
static let fact = Binomials<Mod<P>>(upto: 2e5)
```

In a single-test program it's the same as `let`. Under `@multitest` it's
how precomputation is shared across tests (§6). It's a general feature,
like a `static` local in C or a lazy static in Rust, not a contest one.
`cp explain` lists every static with its memory.

### 2.4 A library file has only declarations

A file with no top-level statements is a **library**: only `fn`,
`struct`, `enum`, `domain`, `const` and `static let`. That's what `use`
accepts. Its fns can't see any per-run `let`s because it has none, so
library code that needs a problem's data takes it as arguments, usually
generic over the domain:

```
// lib/peel.cp
pub fn peel<V>(deg: [V]Int, adj: V -> Iter<V>) -> List<V> = ...
```

`use` of a program file is an error that names the first statement: "lib.foo
has top-level code (line 3); move it into a fn or a separate file". So a
library can never read input or depend on some problem's `n`.

The mode comes from the file's contents; there's nothing to declare.
Tests inside a library go in `@test fn`s, which `cp fntest lib/peel.cp`
runs and which are stripped when the library is used.

## 3. Where state lives

This is the rule that removes the "forgot to reset a global" bug class, and
makes every function's effects visible from its signature.

### 3.1 Top-level `let` is shared, top-level `var` is not

A top-level `let` is set once and never changes, so any function may read
it. Reading `a` from inside a function is the same as having taken it as
an argument:

```
let a: [I]Int = read()

fn best(l: I, r: I) -> Int = max(a[i] for i in l..=r)    // fine: a is a let
```

A top-level `var` is **not** visible inside functions. A function that
changes state takes it as a `mut` parameter, and the call site says `mut`
too:

```
var total = 0

fn add(x: Int):
    total += x                     // error: `total` is a top-level var
                                   // help: take it as a parameter: fn add(mut total: Int, x: Int)

fn add(mut total: Int, x: Int): total += x
add(mut total, 5)
```

So for any call `f(x, mut y)` you can see everything `f` might change: `y`,
and nothing else. That's what makes parts testable alone (§4).

### 3.2 Nested functions

A function inside a function can read its parent's `let`s, the same rule
as at the top level. To change the parent's `var`s it must be declared
`mut fn`, which is the usual DFS helper:

```
fn depths(g: Graph<V>) -> [V]Int:
    var depth: [V]Int = 0
    mut fn dfs(v: V, parent: ?V):
        for a in g.adj(v):
            if a.to != parent:
                depth[a.to] = depth[v] + 1
                dfs(a.to, v)
    dfs(V.first, none)
    depth
```

Rules for `mut fn`:

- **It's a statement, not a declaration.** It exists from its line on,
  and can use the `var`s declared before it. That's also how you write
  the DFS-over-global-state pattern at the top level: declare the `var`s,
  then a `mut fn` below them, with no need to pass `mut depth` through
  every recursive call.
- **It can't escape:** it can't be stored, returned or passed to a
  function that keeps it. So the borrow checker knows exactly where the
  `var`s can change: at calls to the `mut fn`.
- **While a `mut fn` is being called, its captured `var`s can't be
  touched by anything else,** the same exclusivity as a `mut` argument.
- A plain (non-`mut`) nested fn sees only `let`s. It can't read a
  parent's `var`, because the `var` could change between two calls and
  calls with the same arguments could silently give different results.

### 3.3 There's no `pure`

Earlier drafts had a `pure fn` keyword. It's gone, because the rules above
already make every effect visible except I/O:

- a fn can't see top-level `var`s;
- a `mut` parameter is marked at the declaration and at every call;
- a `mut fn` is marked and can't escape.

The only effect left is reading input and printing, and the tools handle
that by recording it (§4.2), not by forbidding it. So there's nothing to
annotate and nothing to think about.

The one place I/O is still checked is `@memo`, where caching a fn that
reads or prints would give wrong answers:

```
@memo
fn best(v: V) -> Int:
    let x: Int = read()            // error: @memo fn `best` reads input (line 12)
                                   // help: read the values before, and pass them in or use a top-level let
```

The check follows calls: if `best` calls a helper that prints, the error
names the helper and the line. Hover and `cp explain` show "does I/O" on
each fn that does.

---------------------------------------------------------------------------

## 4. Parts: splitting a solution into testable pieces

### 4.1 Any function is a part

Many solutions are a pipeline: compute A, use it to compute B, run C on B
and on B reversed, combine. You write each step as a function, and the
top level as the pipeline:

```
fn dist_from(g: Graph<V>, s: V) -> [V]Int = ...
fn costs(d0: [V]Int, d1: [V]Int) -> Costs = ...
fn greedy(c: Costs, k: Int) -> Int = ...
fn overlap(path: List<V>, c: Costs, k: Int) -> Int = ...

let d0 = dist_from(g, x)
let d1 = dist_from(g, y)
let c = costs(d0, d1)
print(max(greedy(c, k), overlap(path, c, k)))
```

No annotation makes something a part. Earlier drafts had `@stage`; it's
gone, because the tools can work out everything a function touches.

### 4.2 What a part gives you

**What a part touches.** Its inputs are its arguments, the top-level
`let`s it reads, and any bytes it reads from input. Its outputs are its
result, its `mut` parameters after the call, and any bytes it prints. That
list is complete, because there's no other way for a fn to affect or be
affected by the rest of the program.

- `answer` above reads `qs` without taking it as an argument; that's
  still an input. The compiler knows which `let`s each part reads.
- A part that reads or prints (say, a helper that prints its answer) is a
  part too. The tools record the bytes it reads and the bytes it prints
  in local builds, so it can be replayed and compared like any other.

Every tool below works on this list: snapshots save the inputs, the trace
shows inputs and outputs, and brutes get the same inputs (including the
same input bytes) and have all outputs compared, printed text included.

**Its own brute.** Write a slow version and name the part it checks:

```
@brute(overlap)
fn overlap_slow(path: List<V>, c: Costs, k: Int) -> Int = ...
```

By default a part's brute is checked **on real calls**: `cp stress`
generates whole inputs, runs the program, and at each call to `overlap`
also calls `overlap_slow` with the same inputs. That matters because
parts usually have preconditions the types don't state: `answer` only
makes sense when `prev` really is `last_seen(a)`. Random arguments would
violate those and report bugs that don't exist.

When a part has no preconditions beyond its types (a math function,
a data structure operation), you can also fuzz it directly:
`@brute(f, args: random)` makes `cp fntest` call both on random arguments
generated from the parameter types, and parameter `where` clauses narrow
what's generated:

```
@brute(merge, args: random)
fn merge_slow(a: List<Int>, b: List<Int>) -> List<Int>
    where a.is_sorted() and b.is_sorted() = (a + b).sorted()
```

A brute that's slow only needs to run on some calls: on large inputs,
`cp stress` checks a sample of calls rather than every one.

**Locating the bug.** When a whole-program stress test fails, `cp locate`
re-runs the failing input with every part that has a brute, and reports
the first part whose result disagrees, with that part's arguments:

```
$ cp stress d
  fails on input 37 (n = 6)
  first part that differs: overlap (greedy, costs, dist_from agree)
  shrunk to n = 3; saved as i6 and snapshot overlap-1
```

**Tracing.** `cp run d i1 --trace` records every top-level `let` and every
part called from the top level, with inputs and result. Parts called from
inside other parts (a helper called a million times) aren't recorded
unless you ask: `--trace overlap` records every call to `overlap`, and
`--trace-depth 2` goes one level deeper. The HTML walkthrough uses the
top-level parts as its chapters.

**Snapshots.** Any part call can be saved, with its arguments and the
top-level `let`s it reads, and replayed without re-reading input or
re-running earlier parts:

```
$ cp run d i1 --stage overlap-1      # reruns only overlap, with saved arguments
```

So when the bug is in step 3 of 4, you iterate on step 3 alone.

### 4.3 Invariants

When a part's answer must not change under a symmetry, say so, and the
tools test it without needing a brute:

```
@invariant(swap(x, y))           // the two start vertices are interchangeable
fn solve(x: V, y: V) -> Int = ...

@invariant(rev)                  // reversing the array doesn't change the answer
fn count_pairs(xs: [I]Int) -> Int = ...
```

### 4.4 Laws

Operation values passed to structures carry laws (associativity, identity,
commutativity). `cp fntest` checks them on random values and on the values
your program actually builds, so a non-associative "monoid" is found
before it corrupts a segment tree.

---------------------------------------------------------------------------

## 5. Whole-program alternatives

### 5.1 A brute for the whole program

```
@brute(main)
fn slow():
    let n: 1..=8 = read()
    ...
    print(answer)
```

`cp stress` generates inputs from the main program's `read()` types, runs
both, and compares outputs. The brute's own `read()` types can be tighter
(`1..=8`), which limits the generated inputs to what the brute can handle.

`main` here means the whole program: the top-level code. A whole-program
brute is for when parts aren't worth separating, or to check the parts'
composition.

### 5.2 Alternatives that are submitted: subtasks

A brute is an alternative implementation used only for testing. A subtask
solution is an alternative that's also **used in the submission**, when
its condition holds:

```
let n: 1..=2e5 = read()
...
print(solve())

fn solve() -> Int = ...                    // the full solution

@solution(solve, when: n <= 5000)
fn solve_quadratic() -> Int = ...          // used when n <= 5000
```

The submission calls `solve_quadratic` when `n <= 5000` holds on the
actual input, else `solve`. `cp stress` checks the two against each
other on inputs where both apply. This is how you keep a working partial
solution while writing the full one, and it's the same mechanism as
`@brute`, aimed at any part. (The prototype's current form is
`@solution(when: ...)` on whole-program functions; this generalizes it to
any part.)

---------------------------------------------------------------------------

## 6. Multitest

Most problems have many tests per input, so multitest and single-test
programs are written the same way. `@multitest` is a stdlib decorator on
the program file itself: it reads the test count, then runs the file's
body once per test.

```
@multitest(sum: n <= 2e5)
use std.primes

static let sieve = Sieve(2e6)           // once for the whole run

let n: 1..=2e5 = read()                 // from here on, everything is per test
domain V = 1..=n
let a: [V]1..=1e6 = read()

if n == 1:
    print(-1)
    return                              // next test

fn prime_pairs() -> Int = ...           // sees this test's n, V and a

@brute(prime_pairs)
fn prime_pairs_slow() -> Int =
    count((i, j) for (i, j) in V.pairs_lt() if sieve.is_prime(a[i] + a[j]))

print(prime_pairs())
```

The single-test version of this program is the same file without the
first line.

- **Per-test state is the body's locals.** `n`, `V`, `a`, and every `var`
  are fresh each test, so nothing can leak from one test into the next
  and there's no reset code. Domains are per test too.
- **Shared precomputation is `static let`.** Without `static`, the sieve
  would be rebuilt every test; `cp budget` warns when a per-test `let`
  doesn't depend on anything read in the test.
- **`return` ends the current test.**
- **Variants:** `@multitest(until_eof)`, and `@multitest(count: q)` when the
  count isn't the first value.
- **Size budgets:** `sum: n <= 2e5` is used by the generator and by
  `cp budget`'s time estimates.
- **`@memo` tables** declared in the body are per test; `static` ones are
  shared across tests.
- **Tools:** brutes and parts work unchanged, per test. `cp locate`
  reports which test and which fn disagreed first, and shrinks that test
  alone.

This isn't the old per-test scope rule (C8). There's no core syntax for
tests: `@multitest` is ordinary stdlib code wrapping a function body, and
the file is that function body.

## 7. Input

### 7.1 Reading is typed

`read()` reads whatever the declared type says:

```
let n: 1..=2e5 = read()
let a: [V]0..=1e9 = read()                 // n values, each range-checked
let es: [E](u: V, v: V, w: 1..=1e9) = read()
let s: [domain S]Char = read()             // a string, with a fresh domain for its positions
let t: Tree<V> = read()                    // n - 1 edges, checked to form a tree
```

Range types and domains are checked as they're read (in local builds; the
judge build trusts the statement by default). `where` adds checks across
values:

```
let (l, r): (V, V) = read() where l <= r
```

### 7.2 The reads are the input spec

Under `cp stress`, the same `read()` calls generate a random value of the
declared type instead of reading one. So the input format, the
constraints, and the random generator are one piece of code. Generation is
edge-biased: minimum and maximum sizes, all-equal values, extremes of each
range, and for structured types, paths, stars and complete graphs.

### 7.3 Your own input logic

Anything unusual implements the `Read` trait (and `Gen`, for stress
testing), as ordinary code. `@derive(Read, Gen)` covers structs and enums
whose fields are readable:

```
@derive(Read, Gen)
enum Query:
    Add(i: V, x: 1..=1e9)        // input "1 i x"
    Ask(l: V, r: V)              // input "2 l r"
```

---------------------------------------------------------------------------

## 8. Output, and problems with many valid answers

`print` writes its arguments separated by spaces, with a newline; every
string literal interpolates:

```
print("Case #{t}: {ans}")
print(xs, sep: "\n")
```

Output is buffered and flushed at exit, and before every read in
interactive mode.

When many answers are valid, write a checker. Its parameter type is the
output format: the tools parse your program's output with `Read`, the
same way input is parsed. It sees the input `let`s too:

```
@checker(output: ?[V]Color)              // "-1", or one color per vertex
fn valid(c: [V]Color) -> Bool = all(c[e.u] != c[e.v] for e in es)
```

`cp stress` then runs the brute, and:
- if the brute says no answer (`none`, printed as `-1`), your output must
  be `-1` too;
- otherwise your output must parse as a `[V]Color` and pass `valid`.

So the checker only decides "is this a correct answer", and the
existence question is settled by the brute.

---------------------------------------------------------------------------

## 9. Local-only code

`cfg` is a compile-time record of build settings: `cfg.local`, `cfg.debug`,
and your own `-D` keys. This is the general mechanism:

```
if const cfg.local:
    print_table(dp)
```

The stdlib adds two shortcuts:

```
debug(dp[v], best)          // prints "d.cp:41  dp[v] = 7  best = 3" to stderr; stripped from submissions
@local fn dump(g: Graph<V>) = ...     // exists only in local builds; calls are stripped
```

`assert` stays in the submission unless you build with `--release`, since
a crash on the judge is more useful than a wrong answer.

---------------------------------------------------------------------------

## 10. Samples and test files

Samples can be inline:

```
@sample("3\n1 2 3\n", "6\n")
```

or files next to the solution: `d.samples/1.in`, `1.out`, ..., or the
plain `i1`, `i2`, ... scratch files in the problem folder that you already
use (`cp run d i1 i2`). `cp test` runs all samples; stress failures are
saved as the next `iN` with the expected output.

---------------------------------------------------------------------------

## 11. Interactive problems

The interaction is described once, as a `protocol`, and the judge's side
is an ordinary struct:

```
protocol Guess:
    @wire("?") fn ask(x: 1..=1e9) -> Ordering
    @wire("!") @final fn answer(x: 1..=1e9)
    limit ask: 30

@interactor(Guess)
@derive(Gen)                         // the hidden value is generated
struct Judge(secret: 1..=1e9):
    fn ask(mut self, x: 1..=1e9) -> Ordering = x <=> self.secret
    fn answer(mut self, x: 1..=1e9) -> Bool = x == self.secret

let judge = Guess.connect()
judge.answer(first_true(1..=1e9, x => judge.ask(x) != Less)!)
```

The solution is still ordinary top-level code; `Guess.connect()` is where
it gets the other side. `@final` marks the call that ends the interaction,
and the interactor's version returns whether the answer is right.

- On the judge, `judge.ask(x)` prints `? x`, flushes and reads the reply
  (stdin/stdout problems), or calls the grader's real function (IOI-style
  problems, where the top-level code becomes the graded function). The
  wire format comes from `@wire` and the types.
- Locally, `judge` is a `Judge` value, so `cp run` and `cp stress` play
  the whole interaction in one process, count queries against the limit,
  and print the call log for a failing case.
- `@interactor(Guess, adaptive)` lets the judge choose its answers to keep
  as many hidden states possible as it can, which finds worst cases a
  random secret misses. Details: plan/16-io.md.

---------------------------------------------------------------------------

## 12. Libraries and modules

```
use std.segtree                  // the stdlib
use lib.treap                    // your own library folder, configured in cp.toml
use lib.graph.flow as fl
```

- A module is a library file (§2.4); `pub` marks what it exports. `use` can't form a
  cycle: a module and the modules it uses form a tree. (Two modules that
  need each other become one module.)
- **Everything is inlined into the one output file and tree-shaken,** so
  only the functions and generic instances you actually call cost bytes.
  That matters because the submission must stay under 64 KB; `cp build
  --size` lists the bytes per library function and per instance.
- Generic library code is specialized where it's used, so library code is
  as fast as code written in the file.

---------------------------------------------------------------------------

## 13. Versions

The first line of every solution is the compiler stamp, added
automatically: `// cp 0.4.2`. The generated C++ carries the same stamp and
a hash of the source. Old solutions keep building with the compiler they
were written for (`cp build --compiler 0.4.2`), and library versions are
snapshotted the same way, so changing your library doesn't silently break
or change old solutions. Details: plan/40-tooling.md.

---------------------------------------------------------------------------

## 14. What the submission contains

`cp submit-file d` produces one C++ file containing:

- the top-level code as `main`, or as the graded functions for IOI-style
  problems, or the selected `@solution`;
- the parts and helpers it calls, and only the library code they use;
- the stamp and source hash as a comment.

It drops brutes, checkers, interactors, samples, `debug`, `@local`
functions, invariants and law tests. Nothing in the testing layer costs
bytes or time on the judge.

---------------------------------------------------------------------------

## 15. Open questions

1. **Top-level `var` at all?** I'd allow it for simple scripts, but keep
   it invisible to functions. The stricter option is "all mutable state
   lives in a function".
2. **Inline `@sample` or files only?** I'd support both.
3. **`debug(x)` in `submit-file`:** silently removed, or a warning? I'd
   remove it silently.
4. **Part names for snapshots:** `fn-callindex` (`overlap-1`), or let you
   name a call? I'd start with the automatic name.
5. **Recording I/O in local builds:** the cost is a copy of the bytes a
   part reads and prints, only when a tool needs it (stress, trace,
   snapshots). Is recording only top-level part calls by default enough?
   (I think so.)
