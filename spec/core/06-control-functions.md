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
  signature (decision 127). Fns of one name whose required labels (the
  named-only ones with no default) differ are overloads: a call takes the
  one whose labels it gives, a call fitting none is the named-argument
  error, and two of one label set clash (E0702). Operators are functions:
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
fn go(v: V):  // nested fn: recursion by name
    seen[v] = true
    for u in g.next(v) if not seen[u]: go(u)

let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)  // recursive lambda
xs.sort(by: p => (p.x, desc(p.y)))
let drop = (v: Int, a: Int) => cnt[a] -= 1  // body may be an assignment
xs.sort(by: x =>                            // block lambda
    let k = key(x)
    (k.0, -k.1)
)
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
- `yield from e` in a `-> yields T` body is exactly `for x in e: yield x`.
  `e` is any `Seq` of `T`, a generator value included; `break`, `return`
  and steering (R06.3.4) act as in that loop, and a recursive generator
  delegating to itself follows R06.3.3. `from` followed by `.`, `(` or
  `[` is the name `from`, so a list or parenthesised operand is bound
  to a name first (decision 668). (R06.3.6)
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
- `@laws(sample: n)`: stress sampling of a type's laws. (R06.4.13)
- `@tag(lit)` on an enum variant: the token it reads and prints as
  (chapter 2).
- `@droppable`: the result may be dropped without the unused-result
  lint. Dropping a non-unit result is that lint (a warning), and
  `let _ = f()` is the same drop and gets the same lint; to drop one on purpose, write
  `with lint(discarded_result: allow)`. Dropping a `?T` is an error (see
  R02.4.7). (R06.4.8)
- `@from_op` on a struct's static fn (std's `Monoid.of`): a bare operator
  (`+`, `max`) passed where that struct is wanted, or for a bound the struct
  has an impl of, becomes a call of the fn (R08.1.9).
- `@memo_table(word)` on a struct (std's `HashMap`): the table a
  `@memo(word)` fn keeps, `T<K, R>` for key `K` (the one parameter's type,
  else the tuple of them) and result `R`, built by `T<K, R>()`. `@memo`
  reads it with `get(k) -> ?R` and keeps a result with `set(k, r)`; its
  bounds on `K` are the key's (R06.4.1). `f.clear()` empties
  it by building a fresh `T<K, R>()`.
- `@same_keys` on a fn (std's `zip`): its arguments that are arrays over
  domains must share one domain, so arrays over two domains are a type
  error at the call (R04b.1.3).
- `@role(recipe)` on a struct (std's `Gen<T>`, §11.6): the value of a `-> yields T`
  call made outside a `for` head, holding the call's fn and evaluated
  arguments in its one fn-typed field; a `for` over it runs that call
  (R07.6.2).
- `@index` on a fn (core's slicing of an `Indexed` by a `Range` of its
  keys): what `s[r]` calls when `r` is not a key of `s`, so `s[l..r]` is
  that fn's `Slice<S>` (R07.4.17).
- `@structural` on a trait (std's `Hash`): a tuple, record, optional,
  array or struct has it when each part does, a position when `Int` does,
  and a built-in scalar only through its own `impl` (so `F64`, with none, is
  not `Hash`). (R06.4.14)
- `@word(name)` on a std struct: `name(args)` is a call of its
  constructor (std's `@word(prefix)` on `Prefix`). (R06.4.15)
- `@member(name)` on a std struct `S<B>`: `D.name` is a domain whose
  positions are the values of `S<D>` (std's `@member(subsets)` on `Subset`,
  so `V.subsets` is the domain of `V`'s subsets). (R06.4.16)
- Markers, each on the one declaration it names: `@operator("op")` (the trait an operator calls,
  through its first declared fn: `"=="` is `Eq`'s `eq`, `"<"` is `Ord`'s
  `lt`, `"[]"` is the read `s[k]` calls and `"[]="` the write `s[k] = v`),
  `@literal("[]")` and `@literal("[;]")` (the traits a list literal and an
  array fill build through, by their fns in declared order, R01.8.8),
  `@derive` (a trait every struct whose fields all have it gets field by
  field: its one fn on each field, as `Clone`'s; or, for a fn that makes a
  `Self` from a first parameter `x`, `x.f<F>()` at each field's type `F`,
  as `Read`'s `input.read<F>()`), `@coerce` (an implicit conversion fn), `@pattern` (the fn a pattern
  over its type calls). The check an `Int` converted to an enum goes
  through, the trait every struct gets through its fields, the
  head-and-tail struct those fields pack into, and the structs and fns a
  literal or a type's syntax is, `(a, b)`, `[a, b]`, `"s"`,
  `[D]T`, `[k: v]`, `lo..hi`, `"{x}"` and a domain's cuts among them, are
  the roles of §11.6.
  Core declares each with them, and any module may: a
  second declaration of one marker, or of one `@operator` with the same
  method, is E0202. (R06.4.12)
- `@input` on a fn: a value it gives is input, so a range type it fills
  is the input's contract, R0502 when broken, not a range store (std's
  `read`, R10.2.2). `@fast(op)` on a fn with a body: the engines and C++ run
  the IR operation `op` for a call, and the reference interpreter runs the
  body, which is what `op` must agree with (std's `@fast(popcount)`). Any
  module may use either, on any number of fns; an `op` the IR does not
  declare is refused when the fn is lowered. (R06.4.12)
- Documented where they act: `@bits` on a field and `@law` (chapter 5),
  `@on_unimplemented`, `@was(..)`, `@aka(..)` and `@stress(kind)` (chapter 12). Any
  other `@name` is E0107, naming the nearest decorator, so no later pass
  meets one. (R06.4.11)
- `@lint(name, from: "<code>", to: "<code>")` on a `pub fn` or a struct's
  or enum's method: a lint the
  library declares, so its users are told to call the fn where they write
  the code it replaces. `from` and `to` are plain strings, each one
  expression or statement or a run of consecutive statements in one block
  (lines joined by `\n`); `from` matches only the whole run and is
  replaced by `to`, its later lines indented as the run is; a hole binds the same
  tokens across the run. The fn's parameter names are the holes: in
  `from` a hole matches any expression (a bracketed one with its
  brackets) or type (`C` in `Fenwick<C, Int>()`) or, where `from` writes
  it as a `let`, `var` or `for` pattern, any pattern there (`let v: T =
  read()` matches `let (u, w): (Int, Int) = read()`), a hole written twice
  matches the same tokens twice and only
  an expression that calls nothing, and every other name matches itself,
  except names `from` binds (lambda parameters, a `for`, `let` or `var`
  variable or each name of its tuple, a `fn` it declares and that fn's
  parameters), which match any name consistently. A hole that stands as a
  whole statement of a block (`body` in `if c:\n    body`) matches the
  rest of that block's statements, and the fix moves them with it, at the
  site's indentation. An `if` `from` matches one branch of an `if` with
  an `elif` or `else` too, an `elif` as its `if`; the fix keeps the other
  branches. A finding names the lint (`warning[name]`),
  is in the `idiom` group, warns by default, and `with lint(name: allow)`
  turns it off; its fix is `to` with each hole replaced by what it matched,
  written over the matched code up to its last token.
  `holes: "a, b"` names more holes than the fn's parameters (a domain or a
  bound the call no longer needs); `to` may drop one of those only when it
  matched an expression that calls nothing, or when the lint has no fix.
  A hole after `.` matches a field or method name (`t[v].kid[c]`). Lines `from` writes
  differently do not match one line of code twice (a mirrored write's
  `g[u].push(v)` then `g[v].push(u)`, not a self-loop's `g[u].push(u)` twice). `when: "b >= 1"` keeps a
  finding only where each listed hole (compared with `>=`, `>`, `<=` or `<`
  and an integer, comma-separated) is proven in bounds: it matched an
  integer literal, a constant expression (`10**18`) whose value does, or
  code whose range type's ends lie in them (`k: 1..=10` for `k >= 1`).
  `also: "t[i] == v\nv == t[i]"` keeps a finding only where one of its
  expressions (one per line) matches somewhere in the file, the holes and
  names `from` bound standing for what they matched; its other holes are
  free. `advice:` in place of `to` names code the finding suggests; it
  need not call the fn nor use every parameter, and is never a fix.
  When `from` declares
  a name with a written type (`var m: Map<K, Int> = Map()`) and `to`
  declares it again with no written type, the fix may change its type, so
  the finding stands only where the rest of the block uses the name through
  a member or an index, never whole (`print(m)`), and the file with the fix
  checks as before. `to` calls the fn by name, as
  a method or getter (`xs.f`), as its compound assignment (`x max= e`,
  R01.6.4), or calls nothing, writes an operator, comparison or `let`
  condition the fn's one-expression body writes, and writes no other that
  `from` does not (`self % d` on `divmod`, whose body is
  `(self / d, self % d)`; `while let w = e, w > y` on `is_some_and`, whose
  body is `if let`), or, on a sequence's `iter`, walks `self` with `for`,
  which runs `iter` (`for x in self`, R07.3.1). On a fn that documents a
  compiler built-in or a field, `builtin: "filled"` lets `to` call that
  built-in or name that field in its place (`[n, m].filled(c)` on a grid
  fn, `t.at` on `Rooted.pos`), since std cannot decorate either. Or `from` calls the fn and `to` does not:
  the lint drops a needless call (`let x = s.last!` then `s.pop()` is
  `let x = s.pop()!`, declared on `last`), its fix titled "drop `f`". A hole
  that `from` evaluates once per item of a comprehension (anywhere in its
  brackets but its first `for`'s iterable) and `to` evaluates once matches
  only code that calls nothing. A hole `to` writes as a comprehension's
  item (`[e for i in 0..n]`) takes the code bare unless it has its own
  `for`, `if` or `=>`. A compound `to` runs its right side before the
  place's indices, so then every hole matches only code that calls
  nothing. A hole whose parameter is written
  as a scalar type (`Int`, `I128`, `U64`, `F64`, `Bool`, `Char`, `Str`)
  matches only code the checker gives that type (a range type is an
  `Int`). One fn's spellings of a name share their sites: the first
  spelling that matches a site takes it. A lint named like a `spec/lints`
  record reports as that record: its code, level and explanation, and
  with no fix when the record gives `no_fix` (its `to` is then only the
  advice). `from` does not match an assigned place (`xs[i] = v`) or the
  base it indexes, and a hole does not match a bare generator (`x * x
  for x in xs` in `sum(..)`), a fold's argument rather than a value. A
  library fn's lint with a fix does not match inside the program's own fn
  of its name (a hand-written `fn starts_with`), whose fix would call
  itself; a method's lint, or one with no fix, does (a hand-written `fn
  find` is `Dsu.find`). A `from` ending in a name (`t.order`) does not
  match the callee of a call (`t.order(v)`). The
  finding spans the matched
  tokens (with no fix, those before a statement hole: the statements it
  matched stay, so the findings in them stand) and labels what `to`'s first hole matched, or with none, the first
  hole `from` calls a method on (the receiver a record's `on` types name);
  with no fix, `to` may drop any hole whatever it matched,
  and a named argument's label (`from:`) is never a hole; where a comment
  falls inside the match it
  has no fix, since `to` would drop the comment.
  A hole `to` uses in the initializer of a name it declares (`let p =
  Prefix([e for i in 0..n])`) does not match code that reads that name.
  `reads: "p[k] -> p.cut(k)"` makes the fix rewrite, too, every later
  use of a name `from` declares, in the rest of its block: each must
  match the read (`k` a hole), else the finding has no fix (a write
  `p[0] = 5`, or `p.len`); `to` or the rewrite calls the fn. For a loop
  variable `from` binds (`for i in 0..self.len`), its uses are those in
  the statement holes, rewritten in the code the holes matched (`reads:
  "self[i] -> x"`). Where `to` drops the name, a use that is no such read,
  or none at all, means the code does not match; nor does it where the
  scope declares again or assigns a name the read's holes matched.
  `each: "x in xs"` spells the name `x` that `to` writes as the singular
  of the one name the hole `xs` matched (`vals` gives `val`); where that is
  no name, or one the file already uses, the finding has no fix.
  A finding inside a wider one is shown with it when `from` still
  matches the wider fix's text, which keeps the code it is about.
  It applies in every file that uses the module, and in the module outside
  the fn; std's apply in every program, as std's names need no `use`. A
  struct's, enum's or trait's method may declare one like a top-level fn;
  a trait's holds for every type that implements it. E0505
  (`bad_library_lint`) when `@lint` is not on a fn, lacks a
  name, `from:` or `to:` (or has both `to:` and `advice:`), when `from`,
  `to` or an `also:` line is not one expression or
  statement or a run of statements, when `reads:` is not two expressions
  around `->`, when `each:` is not a name `to` writes (no hole), `in`,
  and a hole, when `to` drops a parameter `from` matches or uses one it does
  not, when `holes:` names a name neither `from` nor `also:` uses, when `when:` is not
  holes `from` uses compared with integers, when neither `to` (not `advice:`) calls the
  fn (or its `builtin:`) or writes only its body's operators nor `from`
  calls the fn, or when one module declares a
  name twice with the same `from` on one receiver (a name is declared once
  per spelling, on several fns when its fix calls one of them: `min=` or
  `max=`, and once per receiver on methods: `pop_first` on `SortedBag` and
  `SortedSet`). A method's `@lint` whose `from` uses `self`, and whose
  name another type's method also declares, keeps a finding only where the
  checker types what `self` matched as the method's type.
  (R06.4.9)
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
