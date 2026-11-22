# 6. Functions, closures and generators

Status: draft for 0.3.0. Ownership of captures and lends is chapter 5,
which is authoritative; this chapter gives the forms and their meaning.
Sources: archived `design/spec/14-16`, `plan/15-program-structure.md`;
decisions 126, 127, 129, 138, 141, 144.

## 6.1 Functions

```
fn dist(a: P, b: P) -> Int = abs(a.x - b.x) + abs(a.y - b.y)
fn solve(n: Int, mut seen: [V]Bool; limit: Int = 10) -> ?Int:
    ...
```

- One-line `= expr` or a block whose last expression is the value.
  `return` is for early exits.
- Parameters are lent for the call; `mut` parameters may be mutated (or
  drained) and are marked `mut` at the call: `solve(n, mut seen)`. The
  method receiver needs no marker at the call.
- A parameter of a `fn` or a lambda may be a pattern, as in `for`:
  `(mut d, (u, v)) => d.unite(u, v)`. `mut` marks the whole parameter.
- Named and positional arguments, defaults, named-only parameters after
  `;`, and a variadic `..xs: Ts` tuple parameter. Options are always
  named (`by:`, `desc`), never `_by` twins (decision 60).
- No overloading by type: one name and label set has one generic
  signature (decision 127). Operators are functions:
  `fn (a: Frac) + (b: Frac) -> Frac`.
- Functions hoist within their block; every capture must be initialized
  before the first call.

## 6.2 Nested functions and lambdas

Nested `fn`s and lambdas are the same thing for every rule; they differ
only in spelling (decision 144):

```
fn go(v: V):                          // nested fn: recursion by name
    seen[v] = true
    for u in g.next(v) if not seen[u]: go(u)

let fact = n => if n == 0: 1 else: n * fact(n - 1)   // recursive lambda
xs.sort(by: p => (p.x, desc(p.y)))
on_remove: (v, a) => cnt[a] -= 1      // body may be an assignment
by: x =>                              // block lambda
    let k = key(x)
    (k.0, -k.1)
```

- **Captures act as call-time arguments.** A closure holds no lend
  between calls: each call lends what it reads and mutably lends what it
  writes, for the call's duration. So a closure may mutate captured
  `var`s, two closures may capture the same `var`, and the borrow rules
  are checked at each call site (chapter 5).
- A captured variable cannot be moved while the closure is alive, and a
  closure cannot outlive what it captures (it cannot be returned past
  them, stored in an outer `var`, or put in a longer-lived field).
- A `let`-bound lambda may call itself; a `var` binding cannot. When
  recursion leaves the return type open, annotate: `(n: Int) -> Int =>`.
- There is no `pure` and no `mut fn` marker. Purity that a library needs
  (lazy `map`, `@memo`) is a documented contract checked by lints and
  stress, not a type.
- Function types are `(A, B) -> R`. Every call with a known function
  value is specialized; there is no indirect call and no
  `std::function` (chapter 13).

## 6.3 Generators

A generator is a function that pushes elements to its caller's loop body:

```
fn divisors(n: Int) -> yields Int:
    for d in 1..=n.isqrt() if n % d == 0:
        yield d
        if d * d != n: yield n / d

for d in divisors(n): ...
let s = divisors(n).filter(d => d % 2 == 0).sum()
```

- A generator **value** is a recipe: a call with every argument but the
  loop body filled in. Nothing runs until something walks it (`for`, a
  fold, `.collect()`), and each walk reruns it. Walking a value twice is
  legal; walking one that holds a `mut` lend twice is linted.
- A generator value built from lent arguments holds those lends until
  its last use, like a struct built from them (chapter 5).
- The loop body may `break`, `continue`, `return`, use labels and loop
  `else`, even when the generator is recursive (decision 126). A
  non-recursive generator inlines into the loop; a recursive one gets a
  hidden stop path only in specializations whose body can exit.
- A body steers its generator through the yielded item, read after
  `yield` returns: `step.prune()` skips a DFS subtree. Enter/exit walks
  yield events (`Enter(v, via:)`, `Exit(v, via:)`, `Again(from, via)`).
- Generators are push-only; there is no cursor. Lockstep over two
  generators is `List(g)` first, a visible allocation.
- Sequences, views, chains and capabilities are chapter 7.

## 6.4 Decorators

`@d(args)` before an item. The set is closed, each documented by its
library or by this spec:

- `@memo(array)` / `@memo(map)`: cache a function. The storage is
  spelled, never picked (round 3): `array` needs every parameter typed
  by a bounded type (domain, cut, range, `Bool`, enum) and allocates the
  dense table on the first call, `map` uses an open-addressing hash map.
  A parameter bound may mention earlier parameters (`j: 0..=i`); the
  table is then jagged. The cache lives as long as the declaring block
  (per test case at top level below `@!multitest`). A memo function may
  not do I/O or mutate captures; a write to something it captured while
  the cache lives is an error. `cpl explain` shows table bytes.
- `@get`: an O(1) read-only method called without parentheses.
- `@test`, `@alt(f, caps)`, `@gen`, `@check`, `@interactor`: tool
  roles (`tools/stress.md`); the item is otherwise unchanged.
- `@cache` on a struct field: compiler-safe interior mutability with an
  author promise of logical correctness (chapter 5 §10a).
- `@trusted:` blocks and `@cpp` items: the unsafe tier (chapter 9).
- `@grader(...)`: judge entry points (chapter 10).
- `@laws(sample: n)`: stress sampling of a type's laws.
- `@droppable`: the result may be dropped without the unused-result
  lint.

User libraries declare their own decorators as ordinary generic
functions over function types, and their own tool roles with `marker`
(chapter 12, W-list).

## 6.5 Where state lives

- Top-level bindings above `@!multitest` are shared by every test case;
  everything below is per case, including `@memo` caches and fresh
  types.
- A nested function is the way to split a long solution: each chunk is a
  nested fn over the test's state. Any function is a *part* that stress
  and `cpl locate` can test alone (`tools/stress.md`).
