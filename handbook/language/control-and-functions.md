# Control flow and functions

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/06-control-functions.md` |
| Describes | `spec/core/06-control-functions.md`, `spec/conformance/core-06/` |

Why functions, lambdas, generators and decorators in CP Lang look the way
they do. The spec chapter states the rules; this doc gives the problem, the
model and the reasons. Terms such as *lent*, *`mut`* and *trap* are defined
in `spec/GLOSSARY.md`.

## The problem

Contest code is mostly loops, and most loop bugs come from the seams
between a loop and the code that drives it. Three C++ shapes cost real
contestants real minutes:

```cpp
// 1. A library traversal takes a callback, so "stop at the first hit" needs a flag.
bool found = false;
dfs(root, [&](int v) {
    if (found) return;            // the walk keeps going; we only skip work
    if (good(v)) { ans = v; found = true; }
});

// 2. Relaxation tests "did it improve?" with a second comparison.
if (dist[u] + w < dist[v]) { dist[v] = dist[u] + w; par[v] = u; }

// 3. Comparison chains compile and mean something else.
if (0 <= i < n) { ... }           // (0 <= i) < n: always true for n > 1
```

Python fixes the third but not the first: a recursive generator in Python
can stop early, yet costs a frame object per level and is too slow for
\(10^6\) vertices. Kotlin and Ruby fix the first with non-local `return`
from a closure, which gives `return` two meanings depending on where a
lambda was written.

## The model

<!-- given: let (n, lim) = (36, 10) -->
```cp
fn divisors(n: Int) -> yields Int:
    for d in 1..=isqrt(n) if n % d == 0:
        yield d
        if d * d != n: yield n / d

for d in divisors(n):
    if d > lim: break  // break, continue, return, break 'outer, loop else
```

A contestant holds five sentences:

1. A `fn` borrows (lends) its parameters for the call; a parameter it
   changes is marked `mut` both where it is declared and where it is
   passed: `solve(n, mut seen)`.
2. A lambda is an ordinary expression (`x => x * 2`); bound with `let`, it
   may call itself. A nested `fn` and a lambda follow the same rules.
3. A closure holds nothing between calls: each call borrows what it reads
   and writes, so closures can share and mutate a captured `var`.
4. Anything loop-shaped in a library is a generator (`-> yields T`),
   walked by a plain `for`, so every loop control works inside it.
5. One name has one meaning: there is no overloading, and options are named
   arguments (`by:`, `desc`).

## Key decisions

### Library loops are push generators, and `for` keeps its control flow (decision 126)

**Chosen.** A generator runs its body and, at each `yield`, runs the `for`
body. `break`, `continue`, `return`, labelled `break 'outer` and the loop
`else` all work, even when the generator is recursive (R06.3.3). A
non-recursive generator inlines into the loop; a recursive one gets a
hidden stop path only in the specializations whose body can exit.

**Lost.**

| Alternative | Why it lost |
|---|---|
| Callbacks (`dfs(g, s, v => ...)`) | Early exit needs a `found` flag or a `ControlFlow` value per library; stale flags are a bug class |
| Pull iterators (`next()`) | A recursive traversal needs an explicit stack in every library type; generators would cost a hand-written state machine |
| Non-local `return` from closures (Kotlin inline lambdas, Ruby blocks) | `return` would mean two things depending on where the lambda is written; deferred until ports show a need |

**Cost.** Lockstep over two generators is impossible, since each wants to
own the loop. It is spelled `List(g)`, a visible allocation (R06.3.5). The
generator is also steerable without a protocol: the yielded item can carry
methods the generator reads after `yield` returns, as `step.prune()`
skips a DFS subtree (R06.3.4). That is ordinary library code, so a user
generator can do the same, which is the "no privileged stdlib" test.

### A generator value is a recipe (R06.3.1)

**Chosen.** `let g = divisors(n)` stores the function and its arguments and
runs nothing. Each walk (`for`, a fold, `.collect()`) reruns it.

**Lost.** Python's one-shot generator, which silently yields nothing on its
second use: `sum(g); max(g)` fails at run time in Python with an empty
sequence. Here the second walk works, and a lint flags a value walked twice
when it holds a `mut` lend, because the second walk sees what the first
wrote.

### Captures act as call-time arguments (R06.2.1, decision 144)

**Chosen.** A closure holds no borrow between calls. Each call lends what
it reads and mutably lends what it writes for the duration of the call, and
the ownership rules of chapter 5 are checked at each call site.

**Lost.** Rust's model, where a closure that mutates a capture holds a
`&mut` for its whole life, so two closures over one counter are a borrow
error. Contest code does this constantly (`on_add: x => cnt[x] += 1`,
`on_remove: x => cnt[x] -= 1`). The price is R06.2.2: a closure cannot
outlive what it captures, and a captured variable cannot be moved while the
closure is alive. There is no `pure` or `mut fn` marker (R06.2.4): purity
a library needs, such as for `@memo`, is a documented contract checked by
lints and stress, not a type.

### Recursive lambdas, and no overloading (decision 127)

**Chosen.** `let fact = n => if n == 0: 1 else: n * fact(n - 1)` is legal;
a `var` binding cannot refer to itself. It lowers to a self-passing generic
C++ lambda, a direct monomorphized call. One name and label set has one
generic signature (R06.1.6).

**Lost.** Method overloading by argument type. Overloads make errors,
hover and docs ambiguous ("which `find` did you mean?") and break one
spelling per idea. Variants are named arguments instead: decision 60
removed every `_by` twin (`min_by`, `max_by`, `runs_by`, ...) in favour of
`by:` on the base operation, and `.items` is the one spelling of
`enumerate`. Decision 186 shows the knock-on effect: substring search is
`s.search(p)` because `find` already takes a predicate and the language
cannot overload it on a `Str`.

### Lambdas adapt their spelling, never their signature (decision 159, decision 324)

**Chosen.** A lambda with k parameters, passed where a function of one
k-tuple is expected, unpacks it: `a.items.filter((i, x) => ...)`
(R06.1.4). A `fn` passed where fewer parameters are expected fills the
rest from its defaults: `vis.map(yes)` (R06.1.8). Named arguments to a
variadic `..xs` make a named tuple, so `each(sum: +, hi: max)` is a library
function, not a keyword (R06.1.5).

**Why this is not overloading.** The callee's function type is fixed; only
how the lambda is written at the call adapts. The alternative,
`((i, x)) =>`, doubled parentheses at most call sites in the ports.

The variadic rule is the general mechanism behind `each`, so a user
library gets it too (decision 324): positional arguments make a tuple,
named ones a named tuple with its fields in written order, and one call
does not mix the two.

```cp
fn pack<Ts>(..xs: Ts) -> Ts = xs
let p = pack(lo: 1, hi: 9)  // (lo: 1, hi: 9)
let q = pack(1, 9)          // (1, 9)
print(p.hi - p.lo, q.1 - q.0)
```

Lost for it: a separate `each_named` (a `_by`-style twin, which
decision 60 rules out) and a record literal argument,
`each((sum: +, hi: max))`, which costs a pair of parentheses at every use.

### Compound assignment is a statement, except `min=` and `max=` (decision 129)

**Chosen.** `x += y` has no value, like `=`, so there is no `if x = y` bug
and no evaluation-order puzzle. `min=` and `max=` are the exception: as an
expression they are a `Bool`, true when `x` strictly improved (including
from `None`). Relaxation reads as the statement it is:
`if dist[v] min= nd: parent[v] = u`.

**Lost.** Every `op=` returning the new value (`let k = cnt[c] += 1`): handy,
but it puts writes inside expressions. Bump-and-read uses a named mutable
alias instead. `++` and numeric-only unary `+` on non-numbers are rejected
(decision 98): `xs ++ ys` used to lex as `xs + +ys`.

### Comparison chains point one way (decision 130)

**Chosen.** `0 <= i < j <= n` keeps Python's meaning: each operand is
evaluated once and the links are joined by `and`. A chain mixing
directions (`a < b > c`) or containing `==`/`!=` is a compile error
suggesting `and`. C++'s `(a < b) < c` reading is never available.

### Loops take `else` (decision 125)

**Chosen.** `for ... else:` runs when the loop ended without `break`, as in
Python. "Search, and handle not found" otherwise needs a `found` flag,
which is the same stale-state bug site as the callback flag above.

### Decorators are a closed set; the cache's storage is spelled (R06.4)

**Chosen.** `@memo(array)` or `@memo(map)`: the contestant says which
storage, and `array` needs every parameter bounded (a domain, a range, a
`Bool`, an enum), so the table size is known. A bound may mention earlier
parameters (`k: 0..=n`), which makes the table jagged. `@get` marks an
\(O(1)\) member read without parentheses (decision 140, "parentheses mean
work").

**Lost.** A memo that picks its storage silently (a hash map when it can't
prove bounds), which hides a 10x constant factor. User-defined decorator
macros, removed with the other macro facilities; a user library writes a
decorator as an ordinary generic function over function types.

### State above and below `@!multitest` (R06.5.1, decision 385)

Bindings above `@!multitest` are shared by every test case; everything
below, including `@memo` caches, is per case. A shared `var` keeps its
changes across cases, which is how a stamp array avoids an
\(O(\text{size})\) reset per case.

## Worked examples

A generator with a filter fold, a labelled break and a loop `else`:

```cp
fn divisors(n: Int) -> yields Int:
    for d in 1..=isqrt(n):
        if n % d == 0:
            yield d
            if d * d != n: yield n / d

print(divisors(36).filter(d => d % 2 == 0).sum())
'outer: for a in divisors(12):
    for b in divisors(12):
        if a * b == 24 and a < b:
            print(a, b)
            break 'outer
for d in divisors(49):
    if d % 2 == 0:
        print("even divisor", d)
        break
else:
    print("no even divisor")
```

Bellman-Ford relaxation with `min=` as a condition and pattern parameters
in the `for`:

```cp
let inf = 1000000
var dist = [0, inf, inf, inf]
var parent = [-1, -1, -1, -1]
let edges = [(0, 1, 5), (0, 2, 1), (2, 1, 2), (1, 3, 1)]
for _ in 0..3:
    for (u, v, w) in edges:
        if dist[v] min= dist[u] + w: parent[v] = u
print(dist, parent)
```

A closure mutating its capture, and a recursive `let` lambda:

```cp
var cnt = 0
let inc = () => cnt += 1
inc()
inc()
print(cnt)
let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)
print(fact(10))
```

A generator value is a recipe; lockstep goes through `List`:

```cp
fn sq(n: Int) -> yields Int:
    for i in 1..=n: yield i * i
var calls = 0
fn gen(n: Int) -> yields Int:
    calls += 1
    for i in 0..n: yield i
let g = gen(3)
print(calls)
print(g.sum(), g.sum())
print(calls)
let a = List(sq(3))
print(zip(a, [1, 2, 3]).map((x, y) => x - y).sum())
```

Named options, defaults, tuple unpacking and a one-way chain:

```cp
fn f(a: Int, b: Int = 10; scale: Int = 1) -> Int = (a + b) * scale
print(f(1), f(1, 2), f(1, scale: 3))
let ps = [(1, 2), (3, 4)]
print(ps.map((a, b) => a * b).sum())
let xs = [3, 1, 2]
print(xs.sorted(by: x => desc(x)), xs.max(by: x => -x)!)
let x = 5
print(yes(0 <= x < 10))
```

A jagged array memo (\(\binom{30}{15}\)):

```cp
@memo(array)
fn c(n: 0..=30, k: 0..=n) -> Int = if k == 0 or k == n: 1 else: c(n - 1, k - 1) + c(n - 1, k)
print(c(30, 15))
```

# Reference

## Where the precise rules are

| Topic | Spec |
|---|---|
| Function forms, `mut` at the call, defaults, named-only, variadics | `spec/core/06-control-functions.md` §6.1, R06.1.1 to R06.1.8 |
| Lambdas, captures, recursion, function types | §6.2, R06.2.1 to R06.2.5 |
| Generators: recipe, lends, control flow, steering, no lockstep | §6.3, R06.3.1 to R06.3.5; `spec/core/07-sequences.md` "Generators", R07.6.1 to R07.6.4 |
| Decorators | §6.4, R06.4.1 to R06.4.8 |
| Per-case state | §6.5, R06.5.1; `spec/core/10-io.md` R10.1.1 |
| Ownership of captures and lends | `spec/core/05-ownership.md` |
| Conformance cases | `spec/conformance/core-06/` |

## Changes

- 2026-11-23: first version.
