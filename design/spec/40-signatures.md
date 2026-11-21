# 40. Library signatures

**Layer:** library. Status: [proposed]. This chapter is the normative surface: every public item of the
stdlib with its exact signature. Chapters 21–29 explain *why*; where they disagree with this chapter, this
chapter wins. Items marked `[area N]` change shape with the CORE-ALIGN decision for that area.

Conventions: `D: Index` is a domain, `Seq<T>` is anything iterable (stream, view, array, slice, list), and
`?` returns never trap. Every trapping operation names its fallible twin.

## 40.1 Core traits

```cp
trait Index:                                   // 13.2
    fn count() -> Int
    fn ord(self) -> 0..Self.count
    fn from_ord(i: 0..Self.count) -> Self

trait Eq:  fn eq(self, o: Self) -> Bool
trait Ord(Eq): fn cmp(self, o: Self) -> Ordering          // enum Ordering: less, equal, greater
trait Hash: fn hash(self, h: &mut Hasher)
trait Unwrap:
    type Out
    fn unwrap(self) -> Out                     // postfix `!`; traps naming the place: "depth[17] is none"
trait Read: fn read() -> Self                  // 25
trait Print: fn print(self, out: &mut Out)     // 25
```

Arithmetic traits `Add Sub Mul Div Rem Neg Pow BitAnd BitOr BitXor Not Shl Shr`, each
`fn op(self, o: Rhs) -> Out` with `type Out`; `x op= y` uses the same impl (10.5).

## 40.2 Sequences

```cp
impl<T> Seq<T>:
    fn len(self) -> Int
    fn sum(self) -> T                    where T: Add, T: Zero
    fn max(self) -> ?T                   where T: Ord        // none when empty
    fn min(self) -> ?T                   where T: Ord
    fn max_count(self) -> ?(T, Int)      where T: Ord        // value and multiplicity, one pass
    fn min_count(self) -> ?(T, Int)      where T: Ord
    fn max_by<K: Ord>(self, key: T => K) -> ?T               // first maximum
    fn min_by<K: Ord>(self, key: T => K) -> ?T               // first minimum
    fn count(self, where p: T => Bool) -> Int
    fn any(self, p: T => Bool) -> Bool
    fn all(self, p: T => Bool) -> Bool
    fn find(self, p: T => Bool) -> ?T
    fn map<U>(self, f: T => U) -> Seq<U>                     // lazy
    fn filter(self, p: T => Bool) -> Seq<T>                  // lazy
    fn enumerate(self) -> Seq<(Int, T)>                      // integer positions from 0; no fresh domain
    fn scan<A>(self, init: A, f: (A, T) => A) -> Seq<A>      // inclusive; len + 1 values, init first
    fn rscan<A>(self, init: A, f: (A, T) => A) -> Seq<A>     // same, from the right
    fn fold<M: Monoid>(self) -> M                            where T: Into<M>
    fn window_min(self, k: 1..) -> Seq<T>                    where T: Ord   // len - k + 1 values, O(n)
    fn window_max(self, k: 1..) -> Seq<T>                    where T: Ord
    fn pairs(self) -> Seq<(T, T)>                            // adjacent pairs
    fn list(self) -> List<T>
    fn sorted(self) -> [self.len]T                           where T: Ord   // stable
    fn sorted(self, by key: T => K) -> [self.len]T           where K: Ord   // stable
    fn split_by(self, p: T => Bool) -> (List<T>, List<T>)    // (true side, false side), order kept
    fn all_some<U>(self) -> ?[self.len]U                     where T == ?U
```

Sorting is stable everywhere [2impl Q78]. `sorted` returns an anonymous array whose `Cut` is fresh per call
(13.6).

## 40.3 Containers

```cp
struct List<T>                                    // growable [len]T
impl<T> List<T>:
    fn push(var self, x: T)
    fn pop(var self) -> ?T
    fn last(self) -> ?T
    fn clear(var self)
    // indexing as an anonymous array over 0..len; push invalidates slices (16.4)

struct Deque<T>
impl<T> Deque<T>:
    fn push_front(var self, x: T)
    fn push_back(var self, x: T)
    fn pop_front(var self) -> ?T
    fn pop_back(var self) -> ?T
    fn front(self) -> ?T
    fn back(self) -> ?T

struct Heap<T: Ord>(order: Order)                 // enum Order: max_first, min_first; required
impl<T: Ord> Heap<T>:
    fn push(var self, x: T)
    fn pop(var self) -> ?T
    fn top(self) -> ?T

struct Map<K: Ord, V>                             // ordered; HashMap<K: Hash, V> has the same API minus navigation
impl<K: Ord, V> Map<K, V>:
    fn get(self, k: K) -> ?V
    fn contains(self, k: K) -> Bool
    fn remove(var self, k: K) -> ?V
    fn first(self) -> ?(K, V)
    fn last(self) -> ?(K, V)
    fn first(self, after k: K) -> ?(K, V)         // > k; also from: (≥), last(before:) (<), last(upto:) (≤)
    fn pop_first(var self) -> ?(K, V)
    fn pop_last(var self) -> ?(K, V)
    // m[k] reads trap when absent ("key 5 not in m"; use m.get(k));
    // m[k] = v inserts or overwrites; m[k] op= v traps when absent [2impl Q79]

struct Set<T: Ord>                                // Map<T, ()> with insert(x) -> Bool (whether new)

struct Counter<T: Ord>
impl<T: Ord> Counter<T>:
    fn of(self, k: T) -> Int                      // 0 when absent
    fn add(var self, k: T, times: Int = 1)
    fn remove(var self, k: T) -> Bool             // false when absent
    fn min(self) -> ?T
    fn max(self) -> ?T

struct Dsu<V: Index>
impl<V: Index> Dsu<V>:
    fn root(var self, v: V) -> V                  // path halving
    fn union(var self, a: V, b: V) -> Bool        // whether it merged; by size
    fn same(var self, a: V, b: V) -> Bool
    fn size(var self, v: V) -> 1..=V.count

fn compress<T: Ord>(xs: Seq<T>) -> (domain H, Compressed<H, T>)   // .value: [H]T; H(x), H.try(x) -> ?H
```

## 40.4 Algebra and segment trees

```cp
trait Monoid:
    id: Self
    fn op(l: Self, r: Self) -> Self
trait CommutativeMonoid(Monoid)
trait Group(Monoid): fn inv(x: Self) -> Self
trait Idempotent(Monoid)
trait Action<M: Monoid>:
    id: Self
    fn then(t: Self, u: Self) -> Self
    fn apply(t: Self, x: M) -> M
    fn restrict(t: Self, child: M, scope: M) -> Self = t
    fn uniform(t: Self) -> Bool = true

struct Sum<T>(value: T)          // Monoid where T: Add + Zero; Group where T: Neg
struct Max<T>(value: ?T)         // identity none
struct Min<T>(value: ?T)
struct MaxCount<T>(value: ?(T, Int))
struct MinCount<T>(value: ?(T, Int))
struct Add<T>(x: T)              // Action<Sum<T>>, Action<Max<T>>, Action<Min<T>>
struct Assign<T>(x: T)
struct Affine<T>(a: T, b: T)     // x => a * x + b

struct SegTree<D: Index, M: Monoid, A: Action<M> = NoAction>(init: [D]M | M)
impl SegTree<D, M, A>:
    fn fold(self, r: Range<D> | Range<D.Cut> | D) -> M
    fn apply(var self, r: Range<D> | Range<D.Cut> | D, t: A, scope: ?M = none)
    fn prefix_fold(self, start: D.Cut = D.Cut.first, while p: M => Bool) -> (D.Cut, M)
    // t[i] reads a leaf; t[i] = m assigns; t[i] op= x when A == NoAction

struct Fenwick<D: Index, M: Monoid>(init: [D]M | M)
impl Fenwick<D, M>:
    fn add(var self, i: D, x: M)
    fn fold(self, r: RangeTo<D.Cut>) -> M
    fn fold(self, r: Range<D.Cut>) -> M           where M: Group
```

`Range<D> | Range<D.Cut> | D` is shorthand for three impls of one `Slice` trait, not a union type.

## 40.5 Modular arithmetic

```cp
fn Mint(mod: 2..) -> (type Z: Field)              // fresh scoped type, 24.1; a literal modulus specializes
impl Z:
    fn new(x: Int) -> Z                           // written Z(x); floor-reduces negatives
    fn pow(self, e: Int) -> Z                     // negative e uses inv()
    fn inv(self) -> Z                             // traps "inverse of 0"; requires mod: Prime
    fn powers(self, n: 0..) -> [0..n]Z
    fn factorials(n: 0..) -> Factorials<Z>
struct Factorials<Z>
impl Factorials<Z>:
    fn fact(self, k: 0..=self.n) -> Z
    fn inv_fact(self, k: 0..=self.n) -> Z
    fn binom(self, n: Int, k: Int) -> Z           // 0 outside 0 ≤ k ≤ n; traps if n > self.n
```

`a / b` is `a * b.inv()`, so `Z(1) / Z(0)` traps with "inverse of 0" [2impl Q81]. `Int(z)` is the residue.

## 40.6 Graphs and trees

```cp
struct Graph<V: Index, W = ()>                    // CSR; immutable after construction [2impl Q82]
impl Graph<V, W>:
    fn undirected(edges: Seq<(V, V)>) -> Graph<V>
    fn undirected(edges: Seq<(V, V, W)>) -> Graph<V, W>
    fn directed(edges: Seq<(V, V)>) -> Graph<V>
    fn directed(edges: Seq<(V, V, W)>) -> Graph<V, W>
    fn adj(self, v: V) -> Seq<V>
    fn out(self, v: V) -> Seq<(V, W)>
    fn arcs(self) -> Seq<(V, V)>                  // both directions of each undirected edge
    fn components(self) -> (domain C, Partition<V, C>)
    fn two_coloring(self) -> ?(List<V>, List<V>)  // per component call on a Partition part
    fn topo_order(self) -> ?[V.count]V            // none on a cycle
    fn cycle(self) -> ?List<V>

fn bfs<D: Index>(from: D | Seq<D>, next: D => Seq<D>) -> Search<D>
fn dfs<D: Index>(from: D | Seq<D>, next: D => Seq<D>) -> Search<D>
struct Search<D>
impl Search<D>:
    fn dist(self) -> [D]?(0..D.count)
    fn parent(self) -> [D]?D
    fn order(self) -> [self.reached]D             // visit order
    fn path(self, to: D) -> ?List<D>              // from its root, inclusive
    fn root(self) -> [D]?D

struct Tree<V: Index>
fn Tree<V>(edges: Seq<(V, V)>) -> Tree<V>         // traps unless V.count - 1 edges forming a tree
impl Tree<V>:
    fn rooted(self, at: V | Seq<V>) -> Rooted<V>
struct Rooted<V>
impl Rooted<V>:
    fn parent(self, v: V) -> ?V
    fn children(self, v: V) -> Seq<V>
    fn preorder(self) -> [V.count]V
    fn postorder(self) -> [V.count]V
    fn depth(self, v: V) -> 0..V.count
    fn euler(self) -> (pos: [V]Order, span: [V]Range<Order>)   // Order is a fresh domain
```

`Graph`'s two constructor arities are one generic impl over an `EdgeList` trait, not overloading
(14.3) [2impl Q63].

## 40.7 Strings

```cp
struct Str                                         // [len]Byte; indexed by 0..s.len
impl Str:
    fn len(self) -> Int
    fn filled(len: 0.., b: Byte) -> Str
    fn find(self, pat: Str) -> ?(0..=self.len)     // first occurrence
    fn rfind(self, pat: Str) -> ?(0..=self.len)
    fn prefix_function(self) -> [self.len](0..self.len)
    fn z_function(self) -> [self.len](0..=self.len)
    fn borders(self) -> List<1..self.len>
    fn digits(self) -> [self.len](0..=9)           // traps on a non-digit byte
    fn windows(self, m: 1..) -> Seq<Str>
fn SuffixArray(s: Str) -> (domain Rank, order: [Rank](0..s.len), lcp: [Rank](0..=s.len))
```

`+` concatenates two `Str`s into a new one; `==` and `<` compare bytewise.

## 40.8 I/O

```cp
fn read<T: Read>() -> T                         uses io [area 3]
fn print<Ts...: Print>(xs: Ts...)               uses io [area 3, area 9]   // separated by io.sep, then newline
fn yes(b: Bool) -> Answer                       // prints io.answers.0 or .1
struct Wire(error_reply: ?Int = none, limit: ?Int = none)
impl Wire:
    fn ask<T: Read>(var self, q: impl Print) -> T   // prints, flushes, reads; traps on error_reply or limit
    fn say(var self, x: impl Print)                  // prints and flushes
```
