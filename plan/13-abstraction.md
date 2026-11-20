# 1.4 Abstraction: traits, operation values, generics

Status: decided unless a section says *proposed*. Folded from `abstractions.md`
(now in `archive/`); this copy is the source of truth.


Built from `plan/20-stdlib.md` (what must be expressible) and `archive/operations.md`
(operation and ordering values). Goal G8: ~95% of the stdlib is ordinary
code a user could write with the same performance.

# Part 1. Mechanisms

- **M1. Generics:** `<T>`, `<D>`, `<const N>`: compiled per instantiation, zero cost. Needed by everything.
- **M2. Traits:** one canonical instance per type: `Eq Ord Hash Show Read Add Mul`. Needed by sort, maps, printing, ModInt.
- **M3. Operation and ordering values:** many instances per type: `Monoid<T>`, `Order<T>`, ... as values. Needed by segtrees, heaps, sort_by.
- **M4. Laws:** `law` clauses on traits and operation structs, checked by tooling. Needed by algebra, comparators.
- **M5. `if const` and type facts:** compile-time branches on traits, constants and `cfg`. Needed by representation choice.
- **M6. `Buffer<T>` and `@trusted`:** the one raw-storage intrinsic. Needed by List, Heap, Pool, hash tables.
- **M7. Representation traits:** `Niche Index Bits Small Relocate Zero OrdKey Bounded`. Needed by Opt, dense sets/maps, packing.
- **M8. Generators:** `yields T`: lazy sequences compiled to loops. Needed by iteration, sequence views, traversals.
- **M9. Pools and handles:** typed 4-byte handles instead of pointers. Needed by treaps, tries, automata, persistence.
- **M10. Derive:** `Eq Ord Hash Show Read Gen` and M7 traits derived by default. Needed by traces, stress, sets/maps.

Each section below has: what it is, syntax, rules (meaning and checking),
cost (what it compiles to), a stdlib example, errors, and open questions.

## M1. Generics

**What.** Code parameterized over types, domains and constants, compiled
once per concrete instantiation.

**Syntax.**
```
fn max_by<T>(xs: List<T>, key: T -> Int) -> ?T
fn count<D>(a: [D]Bool) -> Int          // D is a domain parameter
fn sum(xs: List<T>) -> T where T: Add   // single capital letter: implicit <T>
struct Matrix<const N: Int, T>(a: [0..N, 0..N]T)
```

**Rules.**
- Parameters: types (`T`), domains (`D`, anything that is `Index`), and
  `const` values of scalar types (`N: Int`, `M: Int`).
- Bounds (`where T: Ord`) are checked **at the definition**, not at the use
  site. Inside the body, only operations granted by the bounds are allowed.
  This is the difference from C++ templates: errors appear in the generic
  code's own terms, once, instead of at every instantiation.
- Type arguments are inferred at call sites; explicit form `f<Int>(...)`.
- No runtime polymorphism (no trait objects). For "one of several kinds",
  use an enum.

**Cost.** Monomorphized: each instantiation is a separate C++ fn, fully
inlined where small. To protect the 64 KB limit, instantiations whose
types have the same layout (e.g. `List<V>` and `List<E>`, both 4-byte
indices) share one copy. The size report lists instantiations by size.

**Example.** `SegTree<D, T>` is generic over its index domain and value
type; one definition covers `[V]Int` and `[Q](min: Int, cnt: Int)` trees.

**Errors.**
```
error: `<` needs T: Ord
  --> lib.cp:4:12
4 |     if x < best: best = x
  |          ^ T is only known to be Eq here
help: add the bound:  fn min_of<T>(xs: List<T>) -> T where T: Ord
```

**Open.** Whether to allow `const` parameters of non-scalar types (e.g. a
const `Monoid` value as a type parameter; see M3 for why we may not need
it).

## M2. Traits

**What.** A named set of operations that a type has **one canonical
version** of: ordering, equality, hashing, printing, reading, arithmetic.

**Syntax.**
```
trait Ord: Eq
    fn lt(a: Self, b: Self) -> Bool
    fn le(a: Self, b: Self) -> Bool = not lt(b, a)   // default method
    law transitive(a, b, c) = not (lt(a, b) and lt(b, c)) or lt(a, c)

impl Ord for Frac:
    fn lt(a, b) = a.p * b.q < b.p * a.q

trait Niche:
    const NONE: Self                                  // associated constant
```

**Rules.**
- Operators are trait methods: `<` is `Ord.lt`, `+` is `Add.add`, `==` is
  `Eq.eq`, `{x}` in strings is `Show`.
- **Coherence:** `impl Tr for T` may live only in the module that defines
  `Tr` or `T`. So there is at most one `Ord` for `Frac` in a program, and
  two libraries can't disagree.
- Most traits are derived (M10); you write an impl only to override.
- Static dispatch only. Associated constants and default methods are
  allowed; associated types are not, until a real need appears.

**Cost.** None: calls resolve at compile time and inline.

**Example.** `ModInt<M>` implements `Add Sub Mul Div Eq Show Read Zero`,
so `Sum<ModInt<M>>`, `Matrix<N, ModInt<M>>` and printing all work.

**Errors.**
```
error: Edge has no Ord, needed by Set<Edge>
help: derive it (fields compared in order: u, v, w), or pass an order:
      Set<Edge>(by: e => e.w)
```
```
error: impl Ord for Edge must live in edge.cp (where Edge is defined)
note: for a one-off ordering, use an Order value: Order.by(e => e.w)
```

**Open.** Should `Show` have a separate "debug" form (for traces) and
"output" form (for `print`)? Leaning yes: `Show` for output, `Debug`
derived always.

## M3. Operation and ordering values

**What.** Behaviour a type can have **many versions of** (sum, max, xor;
ascending, descending, by key) is passed as an ordinary value: a struct of
functions and constants.

**Syntax.**
```
pub struct Monoid<T>(id: T, op: (T, T) -> T)
pub struct Group<T>(id: T, op: (T, T) -> T, inv: T -> T)   // a Group is a Monoid
pub struct Action<T, F>(apply: (F, T) -> T, compose: (F, F) -> F, id: F)
pub struct Order<T>(lt: (T, T) -> Bool)

const Sum<T> = Group<T>(id: T.zero, op: (a, b) => a + b, inv: a => -a) where T: Add
const Max<T> = Monoid<T>(id: T.MIN, op: max) where T: Ord + Bounded

var st = SegTree<V, Int>(Max<Int>)
sort(edges, by: e => (e.w, -e.u))     // key fn builds an Order
let o = Order.by((e: Edge) => e.w).rev.then(e => e.u)
```

**Rules.**
- Operation values are values: store them in fields, pass them, build them
  with combinators (`both(Sum, Max)`, `flip(m)`, `sized(m)`, `opt(m)`,
  `Order.rev`, `.then`, `by`).
- **Subsumption:** a `Group<T>` can be passed where a `Monoid<T>` is
  expected; a `Monoid` where a `Semigroup` is. This is structural (it has
  the fields), not inheritance.
- A trait provides the default value: an `Order<T>` param defaults to
  `T`'s `Ord`; `Sum<T>` is built from `T: Add`.
- The lambdas inside are **pure** (no `mut` captures), so structures may
  call them in any order.

**Cost: code is static, data may be dynamic.** The specialization rule:
- Every fn and lambda has its own compile-time identity (like a Rust
  closure type). A function-typed field carries it in its hidden type:
  `Max<Int>` is `Monoid<Int>[op = max]`, `Sum<Int>` is `Monoid<Int>[op = add]`.
- Generic structures and generic fns are compiled once per identity, as
  they are per type. `SegTree<V, Int>(Max<Int>)` inlines `max`;
  `fn solve(m: Monoid<T>)` gets one copy per monoid it is called with.
  This is a language guarantee, not an optimizer heuristic.
- Captured data is stored as fields: `Affine` mod a `p` read from input
  stores `p`, code still static. Compile-time-constant captures are folded.
- Code chosen at runtime (`if k: max else: min`, a list of different fns)
  is a compile error unless the type is written `dyn (T, T) -> T`, which
  opts into an indirect call.
- The price is code size, as with C++ templates: two monoids give two
  copies of `SegTree`. Counted against 64 KB, listed per copy in the size
  report.

```
error: `op` is max on one branch and min on the other; its code isn't known at compile time
help: branch outside the structure: if k: SegTree(Max<Int>) else: SegTree(Min<Int>)
      or declare `op: dyn (Int, Int) -> Int` to allow an indirect call
```

**Example.** `Dsu<V>(data: Monoid<(size: Int, min: V)>)` keeps a size and
a minimum vertex per component with no extra DSU variant.

**Errors.**
```
error: SegTree<V, Int> needs a Monoid<Int>; got Monoid<(Int, Int)>
help: project with map:  Max<Int>.map(...) or change the tree's value type
```

**Open.** Whether `dyn` fn types are needed at all, or the error alone suffices.

## M4. Laws

**What.** Equations an operation must satisfy (associativity, identity,
transitivity, ...), written next to it and checked by tooling.

**Syntax.**
```
pub struct Monoid<T>(id: T, op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a
```

**Rules.**
- A `law` is a fn returning `Bool`, with no I/O, over arbitrary values of its
  parameter types.
- `cp fntest` checks every law of every operation value used by the
  program, with `Gen`-generated values (M10), shrinking counterexamples.
- Debug runs also check laws on **values that actually occur** (sampled, so
  the cost stays small): a wrong `op` is caught on your own input.
- Structures state what they require by parameter type: `Fenwick` takes a
  `Group`, `SparseTable` takes an idempotent `Semigroup`. So "Fenwick on
  max" is a type error, not a wrong answer.
- Laws are never assumed for optimization unless the type says so.

**Cost.** Zero in release builds; laws are stripped.

**Errors.**
```
fntest: Monoid `best` breaks law assoc
  a = (sum: 1, pre: 5), b = (sum: -2, pre: 0), c = (sum: 3, pre: 3)
  op(op(a, b), c) = (sum: 2, pre: 5)   op(a, op(b, c)) = (sum: 2, pre: 4)
  --> d.cp:12  op: (x, y) => (sum: x.sum + y.sum, pre: max(x.pre, y.pre))
help: pre should be max(x.pre, x.sum + y.pre)
```

**Open.** Laws for `Action` (distributivity over the monoid) need values
of three types at once; fine for `Gen`, but the error format needs work.

## M5. `if const` and type facts

**What.** Compile-time branches, so one library definition picks the best
representation or algorithm per instantiation.

**Syntax.**
```
fn grow(mut self):
    var nb = Buffer<T>.new(2 * buf.cap)
    if const T: Relocate:  @trusted: nb.copy_bytes(0, buf, len)
    else:                  @trusted: for i in 0..len: nb.init(i, buf.take(i))

fn mul(a: T, b: T) -> T:
    if const T.bits <= 32: (a * b) as T
    else: mulmod128(a, b)

if const cfg.local: debug(dist)
```

**Rules.**
- Conditions are compile-time facts: trait membership (`T: Index`),
  associated constants (`T.K`, `T.bits`, `T.bytes`), `const` params, and
  `cfg` (`local`, `judge`, `cpp`).
- Both branches are type-checked; inside a branch, the condition is
  assumed (`T: Index` grants `T.K` and `to_idx`). This means a broken
  branch is caught even if your program never takes it.
- Struct fields can be conditional; methods match on the same condition.
- **`if const` is for choices with no trade-off:** memcpy vs element-wise
  moves, zero-allocation vs a fill loop, `mulmod` by bit width. It never
  changes a structure's memory or complexity.
- **Representation choices with trade-offs are separate types you name.**
  `Set<T>` and `Map<K, V>` are always ordered trees. Dense forms are
  explicit: `Mask<D>` (one word), `BitSet<T>`, `[K]?V`. A bitset hidden
  in `Set<V>` would cost 25 KB per set at n = 2e5 and make iteration,
  `min`, `next`, `clear` O(K/64); when a dense set matters it is part of
  the algorithm, so the code should say so.

**Cost.** Untaken branches are removed before C++ is emitted.

**Example.** `[D]T` filled with a value: if `T: Zero` and the value is
zero, the array is allocated zeroed (`calloc`); otherwise a fill loop.
Same memory, same result, no trade-off.

**Errors.**
```
error: T.K needs T: Index
  --> set.cp:3:31
help: check it first:  if const T: Index
```

**Open.** Whether `if const` can test sizes known only at runtime (n from
input). Proposal: no; use a runtime `if` and both representations.

## M6. `Buffer<T>` and `@trusted`

**What.** The single raw-storage intrinsic: a fixed-capacity block of
slots that may be uninitialized. Everything growable is built on it.

**Syntax.**
```
let b = Buffer<T>.new(cap)
b.cap
@trusted:
    b.init(i, x)          // write into an uninitialized slot
    let y = b.take(i)     // move out, slot becomes uninitialized
    b.move(dst, src, k)   // memcpy-like (requires T: Relocate)
b[i]                      // read an initialized slot (safe)
```

**Rules.**
- Use `Buffer` only when slots can be uninitialized (`List` capacity, heap
  and pool free slots, hash-table empties). Fixed-size, fully initialized
  storage is a `List` or `[D]T`, with no `@trusted`.
- `init`, `take` and `move` are allowed only inside `@trusted` blocks.
  `@trusted` is greppable, and `cp explain` lists every one in the program.
- Debug runs keep a shadow "initialized" bit per slot and report misuse
  (reading an uninitialized slot, double init, leaking) at the source line.
- The owner frees the buffer at end of scope; elements still initialized
  are dropped by compiler-generated code. No user destructors.

**Cost.** A pointer and a capacity; operations are raw loads and stores.

**Example.**
```
pub struct List<T>:
    buf: Buffer<T>
    len: Int
    pub fn push(mut self, x: T):
        if len == buf.cap: grow()
        @trusted: buf.init(len, x)
        len += 1
    fn grow(mut self):
        var nb = Buffer<T>.new(max(4, 2 * buf.cap))
        @trusted: nb.move(0, buf, len)
        buf = nb
```

**Errors.**
```
error: buf.init writes uninitialized memory; only allowed in @trusted
runtime error (debug): slot 7 of Buffer<Node> read before init
  --> heap.cp:31:12   call stack: push → sift_up
```

**Open.** Whether `@trusted` should also unlock unchecked indexing
(`b.get_unchecked(i)`), or whether proven bounds (flow analysis) are
always enough.

## M7. Representation traits

**What.** Facts about a type's values that library code uses through
`if const` to pick a better representation. Derived automatically where
possible.

- **`Niche`** — fact: a spare bit pattern exists; unlocks: `?T` with no extra space
- **`Index`** — fact: values map densely onto `0..K` (domains, fieldless enums, small ranges, tuples of these); unlocks: `[T]X` and `[T]?X` (dense maps), `Mask<T>`, `BitSet<T>`, product domains `[(V, Bool)]Int`
- **`Bits`** — fact: fits in k bits (`0..=1e9` → 30, `V` with n ≤ 2e5 → 18); unlocks: explicit `Packed<(V, V)>` in one `U64`, compact heap entries; never applied implicitly (a shift and mask per access)
- **`Small`** — fact: fits in a few words; unlocks: copied freely; others need `.copy()`
- **`Relocate`** — fact: movable with memcpy; unlocks: `List` growth and sorting by memcpy
- **`Zero`** — fact: all-zero bytes are a valid value; unlocks: zero-allocated arrays instead of fill loops
- **`OrdKey`** — fact: order-preserving map to an integer; unlocks: radix sort, coordinate compression, one-compare tuple ordering
- **`Bounded`** — fact: has `MIN` / `MAX`; unlocks: `INF` for any type, identities like `Max<T>.id = T.MIN`

**Syntax.**
```
trait Niche:   const NONE: Self
trait Index:   const K: Int;  fn to_idx(self) -> 0..K;  fn from_idx(i: 0..K) -> Self
trait Bits:    const bits: Int
trait OrdKey:  fn key(self) -> U64       law order(a, b) = (a < b) == (a.key < b.key)
```

**Rules.**
- **Derivation:** domains and fieldless enums get `Index Niche Bits
  OrdKey Bounded Small Relocate`; ranges get them when small enough;
  tuples and structs get `Index` if every field does (K is the product),
  `Bits` as the sum, `Small` if the total fits in 16 bytes.
- `K` may be a runtime value (a domain sized by `n`); it's fixed once the
  domain is declared.
- Hand-written impls are allowed but checked by laws (`from_idx(to_idx(x))
  == x`, the `OrdKey` order law).
- A niche must be a value the type can never hold, so `?T` never confuses
  a stored value with `none`. `Int` is defined as ±(2^63 − 1): `LLONG_MIN`
  is not an `Int` (producing it is an overflow, trapped in debug), so it
  is a real niche and `?Int` stays 8 bytes. `Int.MIN` is `-(2^63 − 1)`, so
  `Max<Int>.id` and `none` never collide.
- Types with no spare value (`U64`, `I32` at full width) store a flag:
  `?U64` is 16 bytes, shown on hover.

**Cost.** None at runtime; they only steer `if const`.

**Example.** DP over states with a product domain:
```
domain Mask = 0..(1 << k)
var dp: [(V, Mask)]Int = INF      // one flat block, index = v * 2^k + mask
dp[(s, Mask.at(1 << s.idx))] = 0
```

**Errors.**
```
error: [(V, Int)]Bool needs (V, Int): Index, but Int is not Index
help: use a bounded range for the second component, e.g. 0..=k
```

**Open.** Whether `Index` on tuples should allow choosing the layout
(row-major vs column-major); default row-major.

## M8. Generators

**What.** Functions that produce a sequence lazily with `yield`, compiled
into the consuming loop. One mechanism for all custom iteration.

**Syntax.**
```
pub fn runs(a: [D]T) yields (l: D, r: D, x: T) where T: Eq:
    var l = D.first
    for i in D:
        if i == D.last or a[i] != a[i.next!]:
            yield (l, i, a[i])
            if let n = i.next: l = n

pub fn submasks(m: U64) yields U64:
    var s = m
    while true:
        yield s
        if s == 0: break
        s = (s - 1) & m

pub fn preorder(t: Tree<V>, v: V) yields V:
    yield v
    for c in t.children(v): yield from preorder(t, c)

for (s, x) in runs(a): ...            // s: Range1<V>
let big = [c for c in t.children(v) if size[c] * 2 > size[v]]
```

**Rules.**
- A generator is consumed by `for`, comprehensions and the adapters (`map`,
  `filter`, `take_while`, `enumerate`, `zip`).
- A generator **cannot be stored or resumed later**; it exists only within
  the loop that consumes it. That's what makes it free. For an external
  iterator, write a struct.
- `break` and `return` in the consumer leave the generator immediately.
- `yield from` delegates to another generator, including recursively.

**Cost.** The consumer's loop body is inlined at each `yield`, so `runs`
compiles to exactly the hand-written loop. Recursive `yield from` compiles
to a recursive fn taking the body as an inlined lambda (no heap frame).

**Errors.**
```
error: generator runs(a) can't be stored in `let it`
help: iterate it directly (for ... in runs(a)) or collect: List(runs(a))
```

**Proposed (decision 4.4, research/RECOMMENDATIONS.md).** `zip` and
storing are allowed for non-recursive generators, which compile to a pull
state machine; recursive ones in those positions are an error. Plain `for`
keeps push lowering. See 1.3 §5.2.

## M9. Pools and handles

**What.** Growing linked structures (treaps, tries, automata, persistent
trees) without pointers: a pool owns all nodes and hands out typed 4-byte
handles.

**Syntax.**
```
struct Node(key: Int, pri: U32, l: ?Nodes, r: ?Nodes)
pool Nodes: Node                 // declares the pool and its handle type
var nodes = Nodes.new(cap: 4e6)  // preallocate
let h = nodes.add(Node(k, rand(), none, none))
nodes[h].l = some(other)
```

**Rules.**
- Handles are a nominal type (`Nodes`), like a domain: a `Nodes` handle
  can't index another pool or a `[V]` array.
- Handles are only created by `add`, so indexing never goes out of range.
  `?Nodes` is 4 bytes (`Niche`).
- Default pools never free (arena). `Nodes.new(free: true)` adds a free
  list; using a freed handle is a debug-run error.
- Persistent structures copy nodes with `nodes.add(nodes[h] with l: x)`.

**Cost.** One flat array of nodes plus a length; a handle is a `U32`
index. Memory equals the C++ array-of-nodes idiom.

**Example.** An implicit treap is `struct Treap<T, F>(m, act)` holding a
`pool` of nodes; split and merge are ordinary recursive fns over `?Nodes`.

**Errors.**
```
error: `h` is a Trie handle, but `nodes` holds Nodes
runtime error (debug): Nodes handle 812 used after free
```

**Open.** Declaring pools inside generic structs (`pool` per
instantiation) needs syntax; proposal: `pool Nodes: Node` inside the
struct body.

## M10. Derive

**What.** Every struct and enum gets the standard traits automatically,
so values can be compared, hashed, printed, read, generated and packed
without boilerplate.

**Rules.**
- Derived by default: `Eq`, `Ord` (fields in order; enum variants in
  declaration order), `Hash`, `Show`, `Debug`, `Read` (fields in order),
  `Gen`, and the M7 traits where they apply.
- `Gen` respects ranges and `where` clauses, so stress inputs and law
  checks only produce valid values.
- Opt out with `@derive(-Ord)`; a hand-written `impl` replaces the derived
  one.
- `Debug` is the trace contract: every value prints with its type, domain
  values with their domain, optionals as `none`, grids as grids, pool
  handles followed into their nodes (with a depth limit).

**Cost.** Only derived impls the program uses are emitted.

**Example.** `Set<Edge>`, `sort(edges)`, `print(e)`, `let e: Edge =
read()` and a stress generator for `[E]Edge` all work after just
`struct Edge(u: V, v: V, w: 1..=1e9)`.

**Errors.**
```
error: Edge can't derive Hash: field `f: Int -> Int` has no Hash
help: @derive(-Hash), or write impl Hash for Edge
```

**Open.** Whether `Ord` should be derived by default or only on request
(Rust requires asking). Leaning default, since CP sorts tuples constantly.

## Putting it together: `SegTree` in user code

```
pub struct SegTree<D, T>(m: Monoid<T>):
    t: List<T>
    size: Int

    pub fn new(m: Monoid<T>) -> SegTree<D, T>:
        let size = bit_ceil(D.K)                       // M7 (D: Index)
        SegTree(m, List.filled(2 * size, m.id), size)

    pub fn set(mut self, i: D, x: T):
        var p = i.to_idx + size
        t[p] = x
        while p > 1:
            p /= 2
            t[p] = m.op(t[2 * p], t[2 * p + 1])         // M3, inlined

    pub fn fold(self, r: Range<D>) -> T:
        var (l, h) = (r.lo.to_idx + size, r.hi.to_idx + size + 1)
        var (a, b) = (m.id, m.id)
        while l < h:
            if l % 2 == 1: a = m.op(a, t[l]); l += 1
            if h % 2 == 1: h -= 1; b = m.op(t[h], b)
            l /= 2; h /= 2
        m.op(a, b)
```
Generic over the domain and value (M1), takes the operation as a value
(M3) whose laws `cp fntest` checks (M4), and uses `Index` (M7), and prints in traces via `Debug` (M10). It needs no
`Buffer` or `@trusted`: storage is fixed-size and fully initialized. Index
checks on `t[2 * p]` are removed where the checker proves `p < size`;
otherwise each is one predictable branch. It compiles to the same C++ as a
hand-written iterative segment tree.

## Graphs and trees

How the mechanisms above combine for the biggest stdlib area. Everything
here is ordinary L5 library code.

### G1. Vertices and edges are domains
```
domain V = 1..=n
domain E = 1..=m
let edges: [E](u: V, v: V, w: 1..=1e9) = read()
```
Per-vertex data is `[V]T`, per-edge data is `[E]T`. `w[u]` with `u: V`
is a type error; that's the main index-safety win for graph code.
Undirected edges have **one** `E` id shared by both directions. Half-edge
ids exist only where they matter (flow, G9).

### G2. The adjacency contract
A graph is anything that answers "arcs out of `v`": a fn `V -> Iter<A>`
where `A` is a record with at least `to: V`. Weighted algorithms also
read `a.w` (or take `w: A -> W`). Arcs from a stored graph also carry
`e: E`.
```
fn bfs<V, A>(adj: V -> Iter<A>, from: Iter<V>) -> [V]?Int     where A has to: V
fn dijkstra<V, A, W>(adj: V -> Iter<A>, from: Iter<(V, W)>) -> [V]?W
    where A has to: V, w: W;  W: Ord + Add
```
`[V]List<V>`, a stored `Graph`, a grid and a lambda over implicit states
all satisfy it, so there is one `bfs`. Views are plain lambdas:
```
let adj = (u: V) yields: if u != dest: yield from g.adj(u)    // nothing leaves dest
let unweighted = (u: V) => g.adj(u).map(a => (to: a.to))
```
**`Iter<T>` means an inlined generator (M8), never a type-erased
iterator.** In a plain loop the producer's loop body is pasted into the
consumer; storing or zipping one (non-recursive only, proposed) makes a
visible-size `Iterator<T>` state machine, never a boxed one. Anything
you want to keep is an explicit `List`. Generators compose without
allocating: `chain(a, b)`, `.map`, `.filter`. `[...] ++ [...]` builds a
`List` and is not free.

**Cost:** zero. `bfs` is compiled once per `adj` (M3), the generator is
inlined, and field bounds like `A has to: V` are compile-time only.

### G3. Storage is explicit
- **`Graph<V, E, W>`** — representation: CSR: `[V]` offsets (`U32`) + arcs `(to, e, w)`; use: edges known up front; the default
- **`AdjList<V, W>`** — representation: `[V]List<(to, w)>`; use: edges added while running

Built with `Graph.directed(edges)` or `Graph.undirected(edges)`; there is
no default direction. `g.transpose()` returns a new `Graph` (it costs a
rebuild, so it is a call, not a view). CSR memory with `V`, `E` 4 bytes
and `w: 1..=1e9`: 12 bytes per arc, same as a tight C++ CSR.

**Cost:** building the CSR is one counting pass over `edges`, O(n + m), and
it's cheaper than `vector<vector<pair>>` (2 allocations instead of n).

### G4. Algorithms
```
bfs(adj, from) -> [V]?Int            bfs_order(adj, from) yields (v: V, d: Int)
bfs01, dijkstra(adj, from) -> [V]?W   bellman_ford -> ?[V]?W  (none: negative cycle)
topo(adj) -> ?List<V>                 (none: cycle)
scc(adj) -> (domain C, comp: [V]C, dag: Graph<C, ...>)
components(g) -> (domain C, comp: [V]C)
bridges(g) -> [E]Bool    articulation(g) -> [V]Bool    bcc(g) -> (domain B, of: [E]B)
mst(edges, n) -> List<E>  two_sat(n, clauses) -> ?[X]Bool   euler_path(g) -> ?List<E>
```
Results are typed by what they index: `scc` returns a **fresh domain**
`C`, so `[C]Int` per-component arrays can't be indexed with a vertex.
Unreachable is `none`, never `1e18`. Implicit graphs over states use
`bfs` if the state type is `Index` (dense `[S]?Int`) and `bfs_sparse`
(returns `HashMap<S, Int>`) otherwise; you choose by name (G9 rule: no
hidden representation switch).

### G5. Trees
```
let t = Tree.from(edges)            // checks n - 1 edges, connected; error names the bad edge
let r = t.root(exit)                // Rooted<V, E, W>
```
`Rooted` gives:
- `r.up: [V]?Arc` (arc to the parent; `none` at the root), `r.depth`, `r.size`,
- `r.children(v) yields Arc`, `r.pre` / `r.post` (iterative, no recursion
  depth limit),
- `r.below: [E]V` (the child end of each edge),
- `r.tin: [V]Pre` where `Pre` is a **fresh domain** for Euler order,
  `r.at: [Pre]V` its inverse, `r.sub(v): Range<Pre>`, `r.is_anc(a, b)`.

Because `Pre` is a domain, `SegTree<Pre, T>` is indexed by `r.tin[v]` and
writing `st.set(v, x)` with a vertex is a type error. The classic Euler
tour bugs (mixing `v` and `tin[v]`, `tout` off by one) can't be written.

**Cost:** `Tree.from` validation and `root` are O(n). `Rooted`'s fields are
computed only if the program uses them: the compiler removes unused fields
of a struct across the whole program, which is strictly better (G9), and
the memory report lists the fields kept. A program that uses only `up` and
`tin` pays for those two arrays.

### G6. Path and subtree structures take operation values
```
Lift<V, T>(r, m: Monoid<T>, val: [V]T)   fold(x, up_to: a) -> T   (bottom to top)
                                         climb(x, while: V -> Bool) -> V
Lca(r)                                   lca(a, b), dist(a, b)
Hld<V, T>(r, m, val)                     path(a, b) -> T, sub(v) -> T, set(v, x)
reroot<V, T>(r, m: Monoid<T>, lift: (T, Arc) -> T, leaf: V -> T) -> [V]T
centroid(t) yields (c: V, part: Sub<V>)   virtual_tree(r, keys) -> (domain K, ...)
```
`fold` documents its order (deepest first), so non-commutative monoids
work; the monoid's associativity law is checked by `cp fntest` (M4).
**Cost:** same tables as hand-written binary lifting; `Lift` stores
`[LOG][V]V` and `[LOG][V]T`, with `m.op` inlined.
`Hld.path` on a non-commutative monoid needs a reverse fold too; it asks
for `Monoid.both(m, m.flip)` instead of silently assuming commutativity.

### G7. Grids
`domain R = 1..=h; domain C = 1..=w; type Cell = (R, C)` is an `Index`
product (M7), so `[Cell]T` is one flat block. `Dir4` / `Dir8` are enums
with `.delta` and `.rev`; `c.step(d) -> ?Cell` returns `none` off the
edge. `grid.adj(open: Cell -> Bool)` is a G2 view, so `bfs` works as-is.

### G8. Flow
`FlowGraph<V>` has its own half-edge domain: `add(u, v, cap) -> Arc`,
`a.rev`, `flow[a]`, `cap[a]`. `max_flow(s, t)`, `min_cut() -> [V]Bool`,
`min_cost_flow(s, t, k)`. `a.rev` replaces `e ^ 1`.

### G9. Rules this section follows
- Direction, storage (`Graph` vs `AdjList`) and dense vs sparse state maps
  are always named in the code, never chosen implicitly.
- Every result is indexed by the domain it is about (`[V]`, `[E]`, `[C]`,
  `[Pre]`).
- No sentinel values: unreachable is `none`, the root's parent is `none`.

### Example: tree (submission 4456164, roads and shops)
A road is removed; from `x`, report `escaped` if `x` can still reach the
exit, else the distance to the nearest shop below the road, or `oo`.
Original: 70 lines with `tin`/`tout`, three `blift` tables and `-1`
parents.
```
let (n, s, q, e) = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V, w: 1..=1e9) = read()
let shops: [1..=s]V = read()
let exit: V = read()

let r = Tree.from(edges).root(exit)
var is_shop: [V]Bool = false
for v in shops: is_shop[v] = true

var near: [V]Int = inf                       // nearest shop in v's subtree
for v in r.post:
    if is_shop[v]: near[v] = 0
    for a in r.children(v): near[v] = min(near[v], near[a.to] + a.w)

struct Seg(len: Int, best: Int)              // a climbed path segment
const Up = Monoid<Seg>(id: Seg(0, inf),
    op: (lo, hi) => Seg(lo.len + hi.len, min(lo.best, lo.len + hi.best)))
let lift = Lift(r, Up, [V] v => Seg(r.up[v].map(a => a.w) ?? 0, near[v]))

for _ in 0..q:
    let (road: E, x: V) = read()
    let low = r.below[road]
    if not r.is_anc(low, x): print("escaped")
    else:
        let best = lift.fold(x, up_to: low).best
        print(if best >= inf: "oo" else: "{best}")
```
The lifting tables, Euler times and `-1` checks are gone. The
non-obvious part (combining path segments bottom-up) is the 2-line
monoid, and its law is tested. `r.below[road]` replaces "swap so `v` is
the deeper end".

### Example: graph (submission 5593301, multi-source Dijkstra with halving)
Original: 85 lines including a hand-written Dijkstra over `vector<vector<pair>>`,
`1e18` sentinels and a cleared adjacency list for `dest`.
```
fn solve(domain V, domain E, k: Int, dest: V, edges: [E](u: V, v: V, w: Int),
         kind: [V]0..=2) -> Float:
    let g = Graph.undirected(edges)
    let adj = (u: V) yields: if u != dest: yield from g.adj(u)
    let reach = bfs(adj, from: [V.first])
    if reach[dest] == none: return -1

    var dist = dijkstra(adj, from: chain([(V.first, 0.0)],
        V.filter(v => reach[v] != none and kind[v] == 0).map(v => (v, 0.0))))

    fn halved(dist: [V]?Float) yields (V, Float):
        for v in V:
            if let d = dist[v]:
                yield (v, d)
                if kind[v] == 2:
                    for a in adj(v): yield (a.to, d / 2 + a.w)

    for _ in 0..k: dist = dijkstra(adj, from: halved(dist))
    dist[dest] ?? -1
```
Removing `dest`'s out-edges is a one-line view instead of mutating the
graph. Sources are a generator, so the halving rule reads as the problem
states it. No `1e18`: unreached is `none`.

### Open
- Children of `v` as a per-vertex domain (`Kids<v>`) so per-child arrays
  in tree DP are index-safe. Needs dependent domains; parked.
- `Tree.from` validation in judge builds is O(n); keep it on?
- Verify the zero-cost claims once generators exist: `bfs` / `dijkstra`
  over a lambda view should compile to the same assembly as hand-written
  C++. Add to the benchmark suite.
- Whether `A has to: V` (structural field bounds) is a general language
  feature or a one-off. It's generally useful, so I'd make it general.

# Part 2. Stdlib by layer

## L0 intrinsics (compiler-provided)
`Buffer<T>`, integer/float primitives, `I128`, bit ops (`popcount`, `clz`,
`ctz`), raw stdin/stdout, hash seed, `@trusted`.

## L1 core
```
Opt<T>, List<T>, Str, Char, ranges a..b / a..=b, domain helpers
trait Eq, Ord, Hash, Show, Read, Gen
trait Niche, Index, Bits, Small, Relocate, Zero, OrdKey, Bounded
struct Order<T>(lt) + by(key), rev, then;  Eq<T>;  Hash<T>
fn sort(mut xs, by:), sort_by, min/max/argmin, sum, count, any, all
fn partition_point(r, pred), lower_bound(xs, x, by:)
```

## L2 algebra
```
struct Semigroup<T>(op)          law assoc
struct Monoid<T>(id, op)         law assoc, ident
struct Group<T>(id, op, inv)     law + inverse
struct Semiring<T>(zero, one, add, mul)
struct Action<T, F>(apply, compose, id)   law compose, distributes
const Sum<T>, Prod<T>, Max<T>, Min<T>, Xor<T>, Gcd<T>, Affine<T>, MinPlus<T>
fn both(a, b), flip(m), opt(s), sized(m)
struct ModInt<const M: Int>  (impl Add Mul Div; M may be runtime: ModInt.dyn(m))
fn pow(x, k), inv(x), Comb<M>(n) -> table of fact/ifact, nCr
```

## L3 collections
```
Set<T>, Map<K, V> (ordered), HashMap, HashSet, Heap<T>(order), Deque<T>
Pool<N> -> Handle,  BitSet<T: Index> (dense; shifts, and/or, count),  Subset<D> (mask domain)
SortedSet<T>(order) with rank / nth  (order-statistics tree)
```

## L4 structures (all take operation values)
```
prefix(xs, g: Group = Sum) -> Prefix<D, T>   .sum(l..=r)
Fenwick<D, T>(m)              add(i, x), prefix(i), range(l..=r) if Group
SegTree<D, T>(m)              set(i, x), fold(l..=r), max_right(l, pred)
LazySegTree<D, T, F>(m, act)  apply(l..=r, f), fold(l..=r)
SparseTable<D, T>(idem)       fold(l..=r) in O(1)
Dsu<V>(data: CMonoid = none)  unite, find, same, size, data(v)
Treap<T, F>(m, act)           implicit: insert, erase, split, merge, reverse
LiChao<D>                     add_line, min_at(x)
WindowFold<T>(m), MonoDeque<T>(order)
```

## L5 algorithms
```
Graph<V> (CSR), Graph<V, W> weighted, GridGraph<Cell>, implicit graphs via fn adj
bfs, bfs01, dfs (yields enter/exit), dijkstra(g, src; order, monoid), bellman_ford,
floyd, topo, scc -> (comp: [V]C, domain C), bridges, articulation, bcc, two_sat,
euler_path, mst, max_flow (FlowGraph with typed edge pairs), min_cost_flow, matching
Tree<V>.rooted(g, r) -> parent, depth, children, order (Euler domain), size
Lca, Lift<V, T>(m), Hld<V, T>(m), centroid, reroot(tree, merge, lift), virtual_tree
strings: hash (Hash61), kmp/prefix_fn, z, aho_corasick (Pool), suffix_array + lcp,
         suffix_automaton (Pool), manacher, eertree
math: sieve, primes, factor, divisors, phi, mobius, gcd/ext_gcd, crt, miller_rabin,
      pollard, Matrix<N, T>(semiring) pow, ntt/convolve, gauss (field), XorBasis
geometry: P<T>, cross, dot, orient, hull, polygon area, segment intersect, angle sort
dp helpers: lis, knapsack, cht, dnc_opt, knuth, aliens, sos (subset sums)
sequence views: runs (with by:), pairs, windows, groups, scan, compress -> domain
```

## L6 program
`@multitest`, `@memo`, `@brute`, `@invariant`, `@checker`, `protocol`,
`@interactor`, `read`/`print`, `debug`, `@get`, `@trace`.

# Part 3. Coverage against the catalogue

- **Arrays and ranges** — covered by: L1, L4, sequence views, M3; gaps: segment tree beats (needs `Beats` value)
- **Sets, maps, heaps** — covered by: L3, M5, M6; gaps: —
- **Graphs** — covered by: L5 graphs, domains, M8; gaps: flow residual API details
- **Trees** — covered by: L5 trees, M3; gaps: link-cut tree (rare; pool-based, later)
- **Strings** — covered by: L5 strings, M9; gaps: suffix tree (rare)
- **Math** — covered by: L2, L5 math; gaps: polynomial log/exp (rare)
- **DP** — covered by: `@memo`, L5 dp helpers, M3; gaps: slope trick (needs a heap-pair value)
- **Geometry** — covered by: L5 geometry; gaps: float/int split; half-plane (rare)
- **Patterns** — covered by: L6; gaps: —

# Part 4. Open questions

1. **`const` generic values:** is `ModInt<const M: Int>` plus `ModInt.dyn(m)`
   the right split for compile-time vs runtime moduli?
2. **Per-vertex domains:** `tree.children(v)` as a domain family is still
   unsolved; falls back to `List<V>`.
3. **Inlining of operation values:** we rely on C++ templates; worth
   confirming on segtree benchmarks before committing.
4. **How big L5 gets:** everything in the catalogue vs a core set plus your
   own library. (I'd ship the core set and treat the rest as library code
   in `std.extra`.)
