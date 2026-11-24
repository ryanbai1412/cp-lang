# 6. Functions, closures and generators

Status: draft for 0.3.0. Ownership of captures and lends is chapter 5,
which is authoritative; this chapter gives the forms and their meaning.
Sources: archived `design/spec/14-16`, `plan/15-program-structure.md`;
decisions 126, 127, 129, 138, 141, 144.

## The model in one paragraph

A `fn` lends its parameters for the call; `mut` marks a mutated
parameter at both the declaration and the call. A lambda is an
expression; bound with `let` it may call itself (decision 127). There is
no overloading: one name, one function. A loop-shaped library facility
is a generator (`yields`), consumed by a plain `for`, so `break`,
`continue`, `return` and labelled breaks work as in any loop
(decision 126). Ownership of captures and lends is chapter 5.

## 6.1 Functions

<!-- given:
  domain V = 0..4
  struct P: x: Int, y: Int -->
```cp
fn dist(a: P, b: P) -> Int = abs(a.x - b.x) + abs(a.y - b.y)
fn solve(n: Int, mut seen: [V]Bool; limit: Int = 10) -> ?Int:
    todo
```

- One-line `= expr` or a block whose last expression is the value.
  `return` is for early exits. (R06.1.1)
- Parameters are lent for the call; `mut` parameters may be mutated (or
  drained) and are marked `mut` at the call: `solve(n, mut seen)`. The
  method receiver needs no marker at the call. A `fn` parameter always has
  a type (E0204 otherwise); only a lambda's parameters take theirs from
  where it is passed, so a signature reads alone. (R06.1.2)
- A parameter of a `fn` or a lambda may be a pattern, as in `for`:
  `(mut d, (u, v)) => d.unite(u, v)`. `mut` marks the whole parameter. (R06.1.3)
- A lambda with k parameters, passed where the callee expects a function
  of one k-tuple, unpacks it: `(V, V).all((u, v) => ...)` and
  `a.items.filter((i, x) => ...)` mean `((u, v)) =>` and `((i, x)) =>`.
  The callee's function type decides; a lambda that wants the whole tuple
  takes one parameter (`p => p.0`). This is not overloading: the
  signature is fixed and only the lambda's spelling adapts (decision 159). (R06.1.4)
- Named and positional arguments, defaults, named-only parameters after
  `;`, and a variadic `..xs: Ts` tuple parameter. Positional arguments to
  `..xs` make `xs` a tuple; named ones make it a named tuple with those
  fields in written order (`each(sum: +, hi: max)` gets `(sum: +, hi: max)`),
  and one call does not mix the two (decision 324). Options are always
  named (`by:`, `desc`), never `_by` twins (decision 60). (R06.1.5)
- No overloading by type: one name and label set has one generic
  signature (decision 127). Operators are functions:
  `fn (a: Frac) + (b: Frac) -> Frac`. (R06.1.6)
- Functions hoist within their block; every capture must be initialized
  before the first call. A `fn` below `@!multitest` is in the per-test
  body and captures that case's names (R10.1.1). (R06.1.7)
- A `fn` passed where a function of fewer parameters is expected takes
  only those, when every omitted trailing parameter has a default; the
  defaults fill them: `vis.map(yes)` is `vis.map(b => yes(b))`. An omitted
  parameter without a default is a type mismatch (E0302). A generic `fn`
  passes the same way: `sorted(by: desc)`. (R06.1.8)

## 6.2 Nested functions and lambdas

Nested `fn`s and lambdas are the same thing for every rule; they differ
only in spelling (decision 144):

<!-- given:
  domain V = 0..4
  struct P: x: Int, y: Int
  let g = Graph.undirected([(V.first!, V.last!)])
  var seen: [V]Bool = false
  var xs = [P(1, 2), P(3, 4)].to_list()
  var cnt = [0, 0, 0].to_list()
  fn key(p: P) -> (Int, Int) = (p.x, p.y) -->
```cp
fn go(v: V):                          // nested fn: recursion by name
    seen[v] = true
    for u in g.next(v) if not seen[u]: go(u)

let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)   // recursive lambda
xs.sort(by: p => (p.x, desc(p.y)))
let drop = (v: Int, a: Int) => cnt[a] -= 1   // body may be an assignment
xs.sort(by: x =>                      // block lambda
    let k = key(x)
    (k.0, -k.1))
```

- **Captures act as call-time arguments.** A closure holds no lend
  between calls: each call lends what it reads and mutably lends what it
  writes, for the call's duration. So a closure may mutate captured
  `var`s, two closures may capture the same `var`, and the borrow rules
  are checked at each call site (chapter 5). (R06.2.1)
- A captured variable cannot be moved while the closure is alive, and a
  closure cannot outlive what it captures (it cannot be returned past
  them, stored in an outer `var`, or put in a longer-lived field). (R06.2.2)
- A `let`-bound lambda may call itself; a `var` binding cannot. When
  recursion leaves the return type open, annotate: `(n: Int) -> Int =>`. (R06.2.3)
- There is no `pure` and no `mut fn` marker. Purity that a library needs
  (lazy `map`, `@memo`) is a documented contract checked by lints and
  stress, not a type. (R06.2.4)
- Function types are `(A, B) -> R`. Every call with a known function
  value is specialized; there is no indirect call and no
  `std::function` (chapter 13). (R06.2.5)

## 6.3 Generators

A generator is a function that pushes elements to its caller's loop body:

<!-- given: let n = 12 -->
```cp
fn divisors(n: Int) -> yields Int:
    for d in 1..=isqrt(n) if n % d == 0:
        yield d
        if d * d != n: yield n / d

for d in divisors(n): todo
let s = divisors(n).filter(d => d % 2 == 0).sum()
```

- A generator **value** is a recipe: a call with every argument but the
  loop body filled in. Nothing runs until something walks it (`for`, a
  fold, `.collect()`), and each walk reruns it. Walking a value twice is
  legal; walking one that holds a `mut` lend twice is linted. (R06.3.1)
- A generator value built from lent arguments holds those lends until
  its last use, like a struct built from them (chapter 5). (R06.3.2)
- The loop body may `break`, `continue`, `return`, use labels and loop
  `else`, even when the generator is recursive (decision 126). A
  non-recursive generator inlines into the loop; a recursive one gets a
  hidden stop path only in specializations whose body can exit. (R06.3.3, permits)
- A body steers its generator through the yielded item, read after
  `yield` returns: `step.prune()` skips a DFS subtree. Enter/exit walks
  yield events (`Enter(v, via:)`, `Exit(v, via:)`, `Again(from, via)`). (R06.3.4)
- Generators are push-only; there is no cursor. Lockstep over two
  generators is `List(g)` first, a visible allocation. (R06.3.5)
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
  the cache lives is an error. `cpl explain` shows table bytes. (R06.4.1)
- `@get`: an \(O(1)\) read-only method called without parentheses. (R06.4.2)
- `@test`, `@alt(f, caps)`, `@gen`, `@check`, `@interactor`: tool
  roles (`tools/stress.md`); the item is otherwise unchanged. The old
  spellings `@alt(main)`, `@brute(main)`, `@brute(f)` and `@checker` are
  E0503, naming `@alt(program)`, `@alt(f)` and `@check`. Under `@!multitest` a `@check` runs
  once per case, its `read()` over that case's input and its output
  over that case's printed lines. (R06.4.3)
- `@cache` on a struct field: compiler-safe interior mutability with an
  author promise of logical correctness (chapter 5 §10a). (R06.4.4)
- `@trusted:` blocks and `@cpp` items: the unsafe tier (chapter 9).
- `@grader(...)`: judge entry points (chapter 10).
- `@laws(sample: n)`: stress sampling of a type's laws.
- `@tag(lit)` on an enum variant: the token it reads and prints as
  (chapter 2).
- `@droppable`: the result may be dropped without the unused-result
  lint. Dropping a non-unit result is that lint (a warning), and
  `let _ = f()` is the same drop and gets the same lint; to drop one on purpose, write
  `with lint(discarded_result: allow)`. Dropping a `?T` is an error (see
  R02.4.7). (R06.4.8)
- `@lint(name, from: "<code>", to: "<code>")` on a `pub fn`: a lint the
  library declares, so its users are told to call the fn where they write
  the code it replaces. `from` and `to` are plain strings: `to` is one
  expression or statement, and `from` is one expression or statement or a
  run of consecutive statements in one block (lines joined by `\n`), which
  matches only the whole run and is replaced by `to`; a hole binds the same
  tokens across the run. The fn's parameter names are the holes: in
  `from` a hole matches any expression (a bracketed one with its
  brackets), a hole written twice matches the same tokens twice and only
  an expression that calls nothing, and every other name matches itself,
  except names `from` binds (lambda parameters, a `for`, `let` or `var`
  variable), which
  match any name consistently. A finding names the lint (`warning[name]`),
  is in the `idiom` group, warns by default, and `with lint(name: allow)`
  turns it off; its fix is `to` with each hole replaced by what it matched.
  It applies in every file that uses the module, and in the module outside
  the fn. E0505 (`bad_library_lint`) when `@lint` is not on a fn, lacks a
  name, `from:` or `to:`, when `from` or `to` is not one expression or
  statement, when `to` drops a parameter `from` matches or uses one it does
  not, when `to` does not call the fn, or when one module declares a name
  twice. (R06.4.9)
- An idiom whose fix is one library call is declared this way beside the
  fn it teaches, not as a lint in the linter: std's own idiom lints are
  `@lint` declarations in `std/`, and the linter keeps only lints a pattern
  can't state (one that reads types or proven facts, or spans more than one
  block). The `rust_idiom_lints` counter in `scripts/debt-ratchet.py`
  counts the rest and is held at 0 (decision 512). (R06.4.10)

User libraries declare their own decorators as ordinary generic
functions over function types, and their own tool roles with `marker`
(chapter 12, W-list).

## 6.5 Where state lives

- Top-level bindings above `@!multitest` are shared by every test case;
  everything below is per case, including `@memo` caches and fresh
  types. A case may change a shared `var`, and the change carries to
  the next case: a stamp array reset only where a case touched it avoids
  an \(O(\text{size})\) reset per case (decision 385). (R06.5.1)
- A nested function is the way to split a long solution: each chunk is a
  nested fn over the test's state. Any function is a *part* that stress
  and `cpl locate` can test alone (`tools/stress.md`).
