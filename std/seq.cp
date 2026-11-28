//! Sequences: reductions, scans, runs, searching, permutations, monotone stacks, and compression.
//!
//! start: `sum`, `max`, `runs`, `compress`, `min`
//! example: `[1, 2, 3].sum(of: x => x * x) == 14`
//!
//! - Reductions & extrema: `prod`, `is_sorted`, `min`/`max` (with `by:`), `argmax`/`argmin` (with `by:`).
//! - Binary & unimodal search: moved to `std.search` (`ternary_max`/`ternary_min`, `RealRange.bsearch`/`peak`/`valley`, `parallel_bsearch`).
//! - Sorted lookup: `first_ge`/`first_gt`/`last_le`/`last_lt` (with optional `by:`), returning `?D` on `[D]T`, `?Int` on `List<T>`, and `?(K, V)` on `Map<K, V>`.
//! - Runs, chunking & windows: `runs` (with `by:` or `link:`), `buckets`, `windows`, `chunks`, `adjacent`, `pairs`, `with_prev`, `cyclic_pairs`, `zip`.
//! - Scans & folds: `scan` (`.rev.scan` for suffixes); the fold of everything but `i` is `op(pre[i.l], suf[i.r])` over `a.scan(op)` and `a.rev.scan(op)`.
//! - Reordering & selection: `reverse`, `rotate_left`/`rotate_right`, `dedup`/`deduped`, `partition`, `select`, `merge_sorted`.
//! - Permutations: `next_permutation`, `prev_permutation`, `inverse`, `compose`, `perm_cycles`, `perm_pow`, `inversions`.
//! - Nearest smaller: `prev_smaller`/`next_smaller` (larger with `by: desc`), `prev_same`/`next_same`, `MonoStack`; sliding windows fold with `a.window_fold(k, op)` or `Window(op)` (`std.window`).
//! - Coordinate compression: `let (domain C, xs) = compress(all)` -> `Compressed<C, T>` (`index_of`, `try_index_of`, `contains`, `first_ge`, `first_gt`, `last_le`, `last_lt`).

use std.algebra.{Monoid, FromOp}
use std.hash.{Hash, HashMap, HashSet}
use std.math.{abs, popcount, bit}
use std.prefix.{Prefix}
use std.range.{Len, RingRange, ends, first_true, last_true}
use std.rand.{shuffle}
use std.raw.{check_key}

/// Every value of `V` once, in some order. `cpl stress` makes and shrinks it as a permutation.
@stress(perm)
pub type Perm<V> = [V]V

/// Product of all elements of `self` (`T.one` when empty). Works on any `Seq` (`Str`, `List`, `[D]T`, `Range`, views) of `Int`, `Mod<P>`, `F64`.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.prod() == 1`
/// traps: never (overflow wraps in release, traps in debug)
/// example: `[2, 3, 4].prod() == 24`
pub fn prod<S: Seq<T>, T>(self: S) -> T where T: One:
    var out = T.one
    for x in self: out *= x
    out

/// Sum of all elements (`T.zero` when empty), on any `Seq` of a type with `+`; `of:` maps each
/// element first (`xs.sum(of: x => x * x)`). \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.sum() == 0`
/// traps: never (overflow wraps in release, traps in debug)
/// example: `[1, 2, 3].sum(of: x => x * x) == 14`
pub fn sum<S: Seq<T>, T, U: Zero + Add>(self: S, of: T -> U = x => x) -> U:
    var out = U.zero
    for x in self: out += of(x)
    out

/// The average of `of` over the elements as `F64`; `None` when empty.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.mean() == None`
/// traps: never
/// example: `[1, 2].mean() == Some(1.5)`
pub fn mean<S: Seq<T>, T>(self: S, of: T -> F64 = x => F64(x)) -> ?F64:
    var total = 0.0
    var n = 0
    for x in self:
        total += of(x)
        n += 1
    if n == 0: return None
    Some(total / F64(n))

/// Which of several equal best elements an extreme returns: `Ties.First` or `Ties.Last` in
/// sequence order. `a.max(by: f, ties: Ties.Last)` is the rightmost maximum.
pub enum Ties = First | Last

/// The least element by `by`, the first on ties (`ties: Ties.Last` for the last); `None` when empty.
///
/// A method call (`s.min()`) picks this fold; a plain call with two values (`min(a, b)`) is the core binary `min`. `a.keys.min(by: i => a[i])`
/// is argmin. \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.min() == None`
/// ties: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`
/// traps: never
/// example: `[3, 1, 1].min() == Some(1)`
/// example: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`
/// example: `[(1, 'a'), (1, 'b')].min(by: .0, ties: Ties.Last) == Some((1, 'b'))`
@lint(chmin_chmax_if, from: "x = min(x, e)", to: "x min= e", holes: "x, e")
@lint(chmin_chmax_if, from: "x = min(e, x)", to: "x min= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if e < x: x = e", to: "x min= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if x > e: x = e", to: "x min= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if e <= x: x = e", to: "x min= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if x >= e: x = e", to: "x min= e", holes: "x, e")
@lint(loop_argmax, from: "var j = self.first!\nfor i in self:\n    if a[i] < a[j]:\n        j = i", to: "var j = self.min(by: i => a[i])!", holes: "a")
@lint(loop_argmax, from: "var j = self.first!\nfor i in self if a[i] < a[j]:\n    j = i", to: "var j = self.min(by: i => a[i])!", holes: "a")
@lint(sentinel_scalar, from: "var x = v\nfor y in self:\n    x = min(x, y)", to: "let x = self.min()", holes: "v", when: "v >= 1000000000")
@lint(sentinel_scalar, from: "var x = v\nfor y in self:\n    if y < x:\n        x = y", to: "let x = self.min()", holes: "v", when: "v >= 1000000000")
@lint(sort_for_extreme, from: "self.sorted()[0]", to: "self.min()!")
@lint(sort_for_extreme, from: "self.sorted().first!", to: "self.min()!")
@lint(sort_for_extreme, from: "self.sorted(by: by)[0]", to: "self.min(by: by)!")
@lint(sort_for_extreme, from: "self.sorted(by: by).first!", to: "self.min(by: by)!")
pub fn min<S: Seq<T>, T, K: Ord>(self: S, by: T -> K = x => x, ties: Ties = Ties.First) -> ?T:
    let last = ties == Ties.Last
    var best: ?T = None
    for x in self:
        if let b = best:
            if by(x) < by(b) or (last and by(x) == by(b)): best = Some(x)
        else: best = Some(x)
    best

/// The greatest element by `by`, the first on ties (`ties: Ties.Last` for the last); `None`
/// when empty.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.max() == None`
/// ties: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`
/// traps: never
/// example: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`
/// example: `[(2, 'a'), (2, 'b')].max(by: .0, ties: Ties.Last) == Some((2, 'b'))`
@lint(chmin_chmax_if, from: "x = max(x, e)", to: "x max= e", holes: "x, e")
@lint(chmin_chmax_if, from: "x = max(e, x)", to: "x max= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if e > x: x = e", to: "x max= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if x < e: x = e", to: "x max= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if e >= x: x = e", to: "x max= e", holes: "x, e")
@lint(chmin_chmax_if, from: "if x <= e: x = e", to: "x max= e", holes: "x, e")
@lint(loop_argmax, from: "var j = self.first!\nfor i in self:\n    if a[i] > a[j]:\n        j = i", to: "var j = self.max(by: i => a[i])!", holes: "a")
@lint(loop_argmax, from: "var j = self.first!\nfor i in self if a[i] > a[j]:\n    j = i", to: "var j = self.max(by: i => a[i])!", holes: "a")
@lint(sentinel_scalar, from: "var x = v\nfor y in self:\n    x = max(x, y)", to: "let x = self.max()", holes: "v", when: "v <= -1000000000")
@lint(sentinel_scalar, from: "var x = v\nfor y in self:\n    if y > x:\n        x = y", to: "let x = self.max()", holes: "v", when: "v <= -1000000000")
@lint(sort_for_extreme, from: "self.sorted().last!", to: "self.max()!")
@lint(sort_for_extreme, from: "self.sorted(by: by).last!", to: "self.max(by: by, ties: Ties.Last)!")
pub fn max<S: Seq<T>, T, K: Ord>(self: S, by: T -> K = x => x, ties: Ties = Ties.First) -> ?T:
    let last = ties == Ties.Last
    var best: ?T = None
    for x in self:
        if let b = best:
            if by(x) > by(b) or (last and by(x) == by(b)): best = Some(x)
        else: best = Some(x)
    best

/// The least and greatest elements by `by` in one pass, the first of each on ties; `None` when
/// empty.
///
/// A method call (`s.minmax()`) picks this fold; a plain call with two values (`minmax(a, b)`) orders the pair. \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.minmax() == None`
/// ties: `[(1, 'a'), (2, 'b'), (1, 'c'), (2, 'd')].minmax(by: .0) == Some(((1, 'a'), (2, 'b')))`
/// traps: never
/// example: `[5, 1, 4].minmax() == Some((1, 5))`
/// example: `[(2, 'a'), (1, 'b'), (2, 'c')].minmax(by: .0) == Some(((1, 'b'), (2, 'a')))`
pub fn minmax<S: Seq<T>, T, K: Ord>(self: S, by: T -> K = x => x) -> ?(T, T):
    var best: ?(T, T) = None
    for x in self:
        if let (lo, hi) = best:
            if by(x) < by(lo): best = Some((x, hi))
            elif by(x) > by(hi): best = Some((lo, x))
        else: best = Some((x, x))
    best

/// How many elements satisfy `p` (every element without `p`).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.count() == 0`
/// traps: never
/// example: `[1, 2, 3, 4].count(x => x % 2 == 0) == 2`
/// example: `[1, 2, 3].pairs().count() == 3`
pub fn count<S: Seq<T>, T>(self: S, p: T -> Bool = _ => true) -> Int:
    var n = 0
    for x in self:
        if p(x): n += 1
    n

/// Whether some element satisfies `p`; stops at the first.
///
/// cost: \(O(n)\), stopping at the first match
/// empty: `let a: List<Int> = []; not a.any(x => x > 0)`
/// traps: never
/// example: `[1, 2].any(x => x > 1) == true`
pub fn any<S: Seq<T>, T>(self: S, p: T -> Bool) -> Bool:
    for x in self:
        if p(x): return true
    false

/// Whether every element satisfies `p`; stops at the first that doesn't.
///
/// cost: \(O(n)\), stopping at the first failure
/// empty: `let a: List<Int> = []; a.all(x => x > 0) == true`
/// traps: never
/// example: `[1, 2].all(x => x > 1) == false`
pub fn all<S: Seq<T>, T>(self: S, p: T -> Bool) -> Bool:
    for x in self:
        if not p(x): return false
    true

/// Whether `self` is non-decreasing (optionally under key `by:`). Works on any `Seq`.
/// A single element is sorted too.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.is_sorted() == true`
/// traps: never
/// example: `[1, 1, 2].is_sorted() == true`
pub fn is_sorted<S: Seq<T>, T, K: Ord>(self: S, by: T -> K = x => x) -> Bool:
    var prev: ?K = None
    for x in self:
        let k = by(x)
        if let p = prev, k < p: return false
        prev = k
    true

// --- Adapters, ends, and sorting (R07.2.8, R07.4) ---

/// A view of the elements satisfying `p`, in order. \(O(1)\) to make.
///
/// Keys survive: each element keeps its source key, so `a.filter(p).items` gives `(i, a[i])`
/// with `a`'s own `i`. `p` runs as the view is walked, once per element per walk.
/// Bidirectional when `self` is.
///
/// cost: \(O(1)\) to make; \(O(n)\) calls of `p` per walk
/// empty: `let a: List<Int> = []; a.filter(x => x > 0).collect() == []`
/// order: source order
/// traps: never (beyond what `p` traps on)
/// example: `[1, 2, 3, 4].filter(x => x % 2 == 0).collect() == [2, 4]`
/// example: `[5, 6, 7].filter(x => x > 5).items.collect() == [(1, 6), (2, 7)]`
pub fn filter<S: Seq<T>, T>(self: S, p: T -> Bool) -> Filtered<S, T> = Filtered(self, p)

/// \(O(1)\) to make. A view giving `f(x)` for each element. Indexed when `self` is, with the same
/// keys: `a.map(f)[i]` is `f(a[i])`, calling `f` on every read (`collect()` once to cache).
///
/// cost: \(O(1)\) to make; one call of `f` per element read
/// empty: `let a: List<Int> = []; a.map(x => x * x).collect() == []`
/// order: source order, source keys
/// traps: never (beyond what `f` traps on)
/// example: `[1, 2, 3].map(x => x * x).collect() == [1, 4, 9]`
/// example: `domain V = 1..=3; let a: [V]Int = [4, 5, 6]; a.map(x => -x)[V(2)] == -5`
pub fn map<S: Seq<T>, T, U>(self: S, f: T -> U) -> Mapped<S, T, U> = Mapped(self, f)

/// \(O(1)\). A view walking `self` backwards. Keys survive: `.items` and `.keys` give source keys,
/// only the walk order changes, so `a.rev[i]` is a compile error and `a.rev.nth(0)` is the last
/// element (decision 496).
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.rev.collect() == []`
/// order: reverse source order, source keys
/// traps: never
/// example: `[1, 2, 3].rev.collect() == [3, 2, 1]`
@get pub fn rev<S: Bidir>(self: S) -> Rev<S> = Rev(self)

/// \(O(1)\). A view of `(key, element)` pairs with the source's own keys.
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.items.collect() == []`
/// order: source order
/// traps: never
/// example: `[5, 6][1..].items.collect() == [(1, 6)]`
@get pub fn items<S: Keyed<K, T>, K, T>(self: S) -> Items<S> = Items(self)

/// \(O(1)\). The first element, `None` when empty. For the first element satisfying a predicate,
/// use `find(p)`; for its key, `keys.find(p)`.
/// Ties don't apply: position decides.
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.first == None`
/// ties: `[(1, 'a'), (1, 'b')].first == Some((1, 'a'))`
/// traps: never
/// example: `[4, 5].first == Some(4)`
@get pub fn first<S: Seq<T>, T>(self: S) -> ?T:
    for x in self: return Some(x)
    None

/// \(O(1)\). The last element, `None` when empty. For the last element satisfying a predicate,
/// use `rfind(p)`.
/// Ties don't apply: position decides.
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.last == None`
/// ties: `[(1, 'a'), (1, 'b')].last == Some((1, 'b'))`
/// traps: never
/// example: `[4, 5].last == Some(5)`
@lint(peek_then_pop, from: "let x = self.last!\nself.pop()", to: "let x = self.pop()!")
@get pub fn last<S: Bidir<T>, T>(self: S) -> ?T:
    for x in self.iter_rev(): return Some(x)
    None

/// The last element satisfying `p`, `None` if None does; walks backwards and stops there.
///
/// cost: \(O(n)\) calls of `p` at most
/// empty: `let a: List<Int> = []; a.rfind(x => x > 0) == None`
/// ties: `[(2, 'a'), (2, 'b')].rfind(p => p.0 == 2) == Some((2, 'b'))`
/// traps: never (beyond what `p` traps on)
/// example: `[1, 4, 2, 5].rfind(x => x % 2 == 0) == Some(2)`
@lint(sentinel_scalar, from: "var x = -1\nfor i in self:\n    if c:\n        x = i", to: "let x = self.rfind(i => c)", holes: "c")
@lint(sentinel_scalar, from: "var x: Int = -1\nfor i in self:\n    if c:\n        x = i", to: "let x: ?Int = self.rfind(i => c)", holes: "c")
pub fn rfind<S: Bidir<T>, T>(self: S, p: T -> Bool) -> ?T:
    for x in self.rev:
        if p(x): return Some(x)
    None

/// Sorts `self` in place by the comparator `cmp`, a strict weak order; `by:` and `cmp:`
/// don't combine (E0310).
///
/// cost: \(O(n \log n)\) calls of `cmp`
/// traps: in `cpl run` and `--checked`, when `cmp` is not a strict weak order
/// example: `var a = [(3, 1), (1, 2), (2, 2)]; a.sort(cmp: (p, q) => p.0 * q.1 < q.0 * p.1); a == [(1, 2), (2, 2), (3, 1)]`
pub fn sort<S: IndexedMut<D, T>, D: Domain, T>(mut self: S; cmp: (T, T) -> Bool):
    sort_cmp(mut self, cmp)

// `sort(cmp:)`'s body, apart so that no key type is solved for it: `T` needs no `Ord`.
fn sort_cmp<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, before: (T, T) -> Bool):
    let n = self.len
    let ks = self.keys
    var a: List<T> = [self[k] for k in ks]
    var b: List<T> = [self[k] for k in ks]
    let in_a = merge_passes(mut a, mut b, n, before)
    var t = 0
    for k in ks:
        self[k] = if in_a: a[t] else: b[t]
        t += 1

/// Sort `self` in place by `by`; descending is a key, `by: x => desc(x)`. Stable: equal keys keep their order.
/// When no single key orders the elements (an exchange argument, an angle), `cmp: (a, b) => Bool` says
/// whether `a` goes before `b`; give `by:` or `cmp:`, not both (decision 87).
/// Works on any `IndexedMut`, including slices: `a[l..=r].sort()`.
///
/// cost: \(O(n \log n)\) comparisons, \(O(n)\) extra space
/// empty: `var a: List<Int> = []; a.sort(); a == []`
/// ties: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`
/// traps: in `cpl run` and `--checked`, when `cmp` is not a strict weak order (`cmp(x, x)`, or both `cmp(a, b)` and `cmp(b, a)`); otherwise never (beyond what `by` or `cmp` traps on)
/// example: `var a = [3, 1, 2]; a.sort(); a == [1, 2, 3]`
/// example: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`
/// example: `var a = [(3, 1), (1, 2), (2, 2)]; a.sort(cmp: (p, q) => p.0 * q.1 < q.0 * p.1); a == [(1, 2), (2, 2), (3, 1)]`
/// example: `var a = [2, 1]; a.sort(cmp: (x, y) => x <= y); a == [1, 2]` (traps R0001)
/// example: `var a = [2, 1]; a.sort(by: x => x, cmp: (x, y) => x < y); a == [1, 2]` (error E0310)
pub fn sort<S: IndexedMut<D, T>, D: Domain, T, K: Ord>(mut self: S; by: T -> K = x => x):
    // One buffer: each key with its position, heap-sorted in place (ties go by
    // position, so the order is stable), then the elements moved by its cycles.
    let n = self.len
    var a: List<(K, Int)> = []
    for k in self.keys: a.push((by(self[k]), a.len))
    var end = n
    var i = n / 2
    while end > 1:
        if i > 0: i -= 1
        else:
            end -= 1
            a.swap_idx(0, end)
        sift_keyed(mut a, i, end)
    for s in 0..n:
        var cur = s
        while a[cur].1 >= 0:
            let next = a[cur].1
            a[cur] = (a[cur].0, -1)
            if next == s: break
            self.swap_idx(cur, next)
            cur = next

/// Sifts `a[i]` down the max-heap `a[..end]` of keys with their positions.
fn sift_keyed<K: Ord>(mut a: List<(K, Int)>, i: Int, end: Int):
    var p = i
    while 2 * p + 1 < end:
        var c = 2 * p + 1
        if c + 1 < end and keyed_before(a[c], a[c + 1]): c += 1
        if not keyed_before(a[p], a[c]): return
        a.swap_idx(p, c)
        p = c

fn keyed_before<K: Ord>(x: (K, Int), y: (K, Int)) -> Bool = x.0 < y.0 or (not (y.0 < x.0) and x.1 < y.1)

/// The bottom-up passes of `sort` over `a`, using `b` as the other buffer; true when the
/// sorted result ends in `a`.
fn merge_passes<E>(mut a: List<E>, mut b: List<E>, n: Int, before: (E, E) -> Bool) -> Bool:
    var in_a = true
    var w = 1
    while w < n:
        if in_a: merge_runs(a, mut b, w, before)
        else: merge_runs(b, mut a, w, before)
        in_a = not in_a
        w *= 2
    in_a

/// One bottom-up pass of `sort`: merges each pair of adjacent sorted runs of length `w` in
/// `src` into `dst`, stably; `before(x, y)` says `x` goes first.
fn merge_runs<E>(src: List<E>, mut dst: List<E>, w: Int, before: (E, E) -> Bool):
    let n = src.len
    var lo = 0
    while lo < n:
        let mid = if lo + w < n: lo + w else: n
        let hi = if lo + 2 * w < n: lo + 2 * w else: n
        var (i, j) = (lo, mid)
        for k in lo..hi:
            let right = j < hi and (
                i >= mid or before(src[j], src[i])
            )
            if right:
                dst[k] = src[j]
                j += 1
            else:
                dst[k] = src[i]
                i += 1
        lo = hi

/// The elements of `self` sorted by `by` or `cmp`: the value form of `sort`. A new `List`, keyed
/// from 0; to keep keys, sort the keys instead (`a.keys.sorted(by: i => a[i])`).
/// Stable: equal keys keep their relative order.
///
/// cost: \(O(n \log n)\) comparisons, \(O(n)\) space
/// empty: `let a: List<Int> = []; a.sorted() == []`
/// ties: `[(1, 'b'), (0, 'z'), (1, 'a')].sorted(by: p => p.0) == [(0, 'z'), (1, 'b'), (1, 'a')]`
/// order: increasing `by` (decreasing with `by: x => desc(x)`), keyed from 0
/// traps: as `sort`
/// example: `[3, 1, 2].sorted() == [1, 2, 3]`
/// example: `[3, 1, 2].sorted(by: x => desc(x)) == [3, 2, 1]`
/// example: `[(3, 1), (1, 2), (2, 2)].sorted(cmp: (p, q) => p.0 * q.1 < q.0 * p.1) == [(1, 2), (2, 2), (3, 1)]`
@lint(argsort_loop, from: "[(xs[i], i) for i in self].sorted()", to: "self.sorted(by: i => xs[i])", holes: "xs")
@lint(argsort_loop, from: "for i in self:\n    p.push((xs[i], i))\nlet s = p.sorted()", to: "let s = self.sorted(by: i => xs[i])", holes: "xs, p")
pub fn sorted<S: Seq<T>, T, K: Ord>(self: S; by: T -> K = x => x) -> List<T>:
    var out: List<T> = self.collect()
    sort(mut out, by:)
    out

/// A new list of the elements ordered by the comparator `cmp`, a strict weak order; `by:` and
/// `cmp:` don't combine (E0310).
///
/// cost: \(O(n \log n)\) calls of `cmp`
/// traps: in `cpl run` and `--checked`, when `cmp` is not a strict weak order
/// example: `[(3, 1), (1, 2), (2, 2)].sorted(cmp: (p, q) => p.0 * q.1 < q.0 * p.1) == [(1, 2), (2, 2), (3, 1)]`
pub fn sorted<S: Seq<T>, T>(self: S; cmp: (T, T) -> Bool) -> List<T>:
    var out: List<T> = self.collect()
    sort_cmp(mut out, cmp)
    out

/// \(O(n)\). A new list of the elements in reverse order; `self.rev` walks them without copying.
///
/// cost: \(O(n)\)
/// empty: `let e: List<Int> = []; e.reversed() == []`
/// order: reverse source order, keyed from 0
/// traps: never
/// example: `[1, 2, 3].reversed() == [3, 2, 1]`
@lint(reverse_index, from: "[self[self.len - 1 - i] for i in 0..self.len]", to: "self.reversed()")
pub fn reversed<S: Bidir<T>, T>(self: S) -> List<T> = self.rev.collect()

/// \(O(n)\). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).
/// Works on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).
///
/// cost: \(O(n)\), \(O(1)\) extra space
/// empty: `var a: List<Int> = []; a.reverse(); a == []`
/// traps: when `range` falls outside the keys
/// example: `var a = [1, 2, 3]; a.reverse(); a == [3, 2, 1]`
pub fn reverse<S: IndexedMut>(mut self: S, range: ?Range<Int> = None):
    var lo = 0
    var hi = self.len
    if let r = range:
        if r.is_empty: return
        lo = r.first!
        hi = r.last! + 1
    var (i, j) = (lo, hi - 1)
    while i < j:
        self.swap_idx(i, j)
        i += 1
        j -= 1

/// \(O(n)\). Drop each value equal to the one before it, in place: a sorted list keeps one of each.
/// Each run of equal values keeps its first.
///
/// cost: \(O(n)\), \(O(1)\) extra space
/// empty: `var a: List<Int> = []; a.dedup(); a == []`
/// ties: `var a = [2, 2, 2, 1, 1]; a.dedup(); a == [2, 1]`
/// traps: never
/// example: `var a = [1, 1, 2, 1]; a.dedup(); a == [1, 2, 1]`
pub fn dedup<T: Eq>(mut self: List<T>):
    var n = 0
    for i in 0..self.len:
        if n == 0 or self[i] != self[n - 1]:
            self[n] = self[i]
            n += 1
    while self.len > n: let _ = self.pop()

/// `self` without each value equal to the one before it: the value form of `dedup`, as `sorted` is of `sort`.
/// Works on any `Seq`. \(O(n)\) time without hashing.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.deduped() == []`
/// order: first of each run, in input order; a new `List`, keyed from 0
/// traps: never
/// example: `[1, 1, 2, 1].deduped() == [1, 2, 1]`
pub fn deduped<S: Seq<T>, T: Eq>(self: S) -> List<T>:
    var out: List<T> = []
    for x in self:
        if out.is_empty or out.last! != x:
            out.push(x)
    out

/// \(O(n)\). The positions holding a new strict maximum, left to right: the first position, then
/// each one greater than everything before it. `record_highs(by: f)` compares `f(x)`.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.record_highs().collect() == []`
/// ties: `[2, 2, 3].record_highs().collect() == [0, 2]`
/// order: increasing keys of `self`
/// traps: never
/// example: `[3, 1, 4, 4, 5].record_highs().collect() == [0, 2, 4]`
pub fn record_highs<D, T, K: Ord>(self: [D]T, by: T -> K = x => x) -> yields D:
    var best: ?K = None
    for i in D:
        let k = by(self[i])
        if (if let b = best: k > b else: true):
            best = k
            yield i

/// Distinct values of `self` in first-seen order (optionally keyed by `by:`).
///
/// Works on any `Seq`. \(O(n)\) expected time, \(O(n)\) space via `HashSet`. Unlike
/// `dedup` / `deduped` (adjacent duplicates only, \(O(n)\) time without hashing),
/// `distinct` drops duplicates anywhere in the sequence.
///
/// cost: \(O(n)\) expected, \(O(n)\) extra space
/// empty: `let a: List<Int> = []; a.distinct() == []`
/// ties: `[(1, 'a'), (2, 'b'), (1, 'c')].distinct(by: .0) == [(1, 'a'), (2, 'b')]`
/// order: first-seen order; a new `List`, keyed from 0
/// traps: never
/// example: `[3, 1, 3, 2, 1].distinct() == [3, 1, 2]`
pub fn distinct<S: Seq<T>, T, K: Hash + Eq>(self: S, by: T -> K = x => x) -> List<T>:
    var seen = HashSet<K>()
    let out: List<T> = [x for x in self if seen.add(by(x))]
    out

/// The values of `self` that `keep` holds for, then the ones it does not, each in the order they came.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.partition(x => x > 0) == ([], [])`
/// order: each list keeps input order
/// traps: never
/// example: `[1, 2, 3, 4].partition(x => x % 2 == 0) == ([2, 4], [1, 3])`
pub fn partition<S: Seq<T>, T>(self: S, keep: T -> Bool) -> (List<T>, List<T>):
    var (yes, no): (List<T>, List<T>) = ([], [])
    for x in self:
        if keep(x): yes.push(x)
        else: no.push(x)
    (yes, no)

/// \(O(n)\) expected, \(O(n \log n)\) worst case, \(O(1)\) extra space. Introselect partitioning `self` in place
/// so `self.nth(k)` holds the element that would sit at `k` in sorted order (by `by:` or `cmp:`).
///
/// Equal elements may land in any order on either side of `k`.
///
/// cost: \(O(n)\) expected, \(O(n \log n)\) worst case, \(O(1)\) extra space
/// empty: `var a: List<Int> = []; a.select(0); a == []` (traps R0401)
/// ties: `var a = [2, 1, 2, 1]; a.select(2); a.nth(2) == 2`
/// traps: when `k` is not in `0..self.len`
/// example: `var a = [5, 1, 4, 2]; a.select(1); a.nth(1) == 2`
pub fn select<S: IndexedMut<D, T>, D: Domain, T, K: Ord>(
    mut self: S,
    k: Int,
    by: T -> K = x => x,
    cmp: ?((T, T) -> Bool) = None,
):
    let n = self.len
    assert 0 <= k < n, "select needs k in 0..len"
    let less = (a: T, b: T) => if let c = cmp: c(a, b) else: by(a) < by(b)
    var (lo, hi) = (0, n)
    var budget = 2
    var m = n
    while m > 1:
        budget += 2
        m /= 2
    while hi - lo > 1:
        if budget == 0:
            heap_select(mut self, lo, hi, k, less)
            return
        budget -= 1
        let mid = lo + (hi - lo) / 2
        if less(self.nth(mid), self.nth(lo)): self.swap_idx(mid, lo)
        if less(self.nth(hi - 1), self.nth(lo)): self.swap_idx(hi - 1, lo)
        if less(self.nth(hi - 1), self.nth(mid)): self.swap_idx(hi - 1, mid)
        let pivot = self.nth(mid)
        var (i, j) = (lo, hi - 1)
        while i <= j:
            while less(self.nth(i), pivot): i += 1
            while less(pivot, self.nth(j)): j -= 1
            if i <= j:
                self.swap_idx(i, j)
                i += 1
                j -= 1
        if k <= j: hi = j + 1
        elif k >= i: lo = i
        else: return

fn heap_select<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, lo: Int, hi: Int, k: Int, less: (T, T) -> Bool):
    let m = hi - lo
    var start = m / 2 - 1
    while start >= 0:
        sift_down(mut self, lo, start, m, less)
        start -= 1
    var end = m - 1
    while end > k - lo:
        self.swap_idx(lo, lo + end)
        sift_down(mut self, lo, 0, end, less)
        end -= 1
    if end >= 0:
        self.swap_idx(lo, lo + end)

fn sift_down<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, lo: Int, from: Int, m: Int, less: (T, T) -> Bool):
    var i = from
    while true:
        var c = 2 * i + 1
        if c >= m: return
        if c + 1 < m and less(self.nth(lo + c), self.nth(lo + c + 1)): c += 1
        if not less(self.nth(lo + i), self.nth(lo + c)): return
        self.swap_idx(lo + i, lo + c)
        i = c

/// \(O(1)\) per step, \(O(1)\) memory. Stable merge of two sorted sequences (keyed by `by:`),
/// yielded lazily, taking from `self` before `b` on ties. Works on any `Positional` inputs
/// (lists, slices, ranges, `.rev` and `.map` views); a generator input is `List(g)` first,
/// since two generators can't be walked in step (R07.6.4).
///
/// Within each input, input order is kept (stable).
///
/// cost: \(O(1)\) per step and \(O(1)\) memory, so a whole walk is \(O(\lvert a \rvert + \lvert b \rvert)\); `.collect()` keeps it as a `List`
/// empty: `let e: List<Int> = []; merge_sorted([1, 2], e) == [1, 2] and merge_sorted(e, [3]) == [3]`
/// ties: `merge_sorted([(1, 'a')], [(1, 'b')], by: .0) == [(1, 'a'), (1, 'b')]`
/// order: non-decreasing by `by`; a generator, rerun on every walk
/// traps: never; unsorted inputs give an unspecified interleaving
/// example: `merge_sorted([1, 3], [2, 3]) == [1, 2, 3, 3]`
pub fn merge_sorted<A: Positional<T>, B: Positional<T>, T, K: Ord>(self: A, b: B, by: T -> K = x => x) -> yields T:
    var (i, j) = (0, 0)
    while i < self.len and j < b.len:
        if by(b.nth(j)) < by(self.nth(i)):
            yield b.nth(j)
            j += 1
        else:
            yield self.nth(i)
            i += 1
    while i < self.len:
        yield self.nth(i)
        i += 1
    while j < b.len:
        yield b.nth(j)
        j += 1

/// \(O(1)\) per window. Lent length-`k` sliding slice views, left to right. Nothing when `k > self.len`.
///
/// cost: \(O(1)\) per window
/// empty: `let a: List<Int> = []; a.windows(2).count() == 0`
/// order: left to right; each window keeps the keys of `self`
/// traps: when `k < 1`
/// example: `[1, 2, 3].windows(2).map(w => w.sum()).collect() == [3, 5]`
pub fn windows<S: Indexed>(self: S, k: 1..) -> yields Slice<S>:
    assert k >= 1, "a window holds at least one element"
    for i in 0..self.len - k + 1: yield piece(self, i, i + k)

/// \(O(1)\) per chunk. Lent non-overlapping slice views of length `k` (the last may be shorter).
///
/// cost: \(O(1)\) per chunk
/// empty: `let a: List<Int> = []; a.chunks(2).count() == 0`
/// order: left to right; each chunk keeps the keys of `self`, the last may be shorter
/// traps: when `k < 1`
/// example: `[1, 2, 3].chunks(2).map(c => c.len).collect() == [2, 1]`
pub fn chunks<S: Indexed>(self: S, k: 1..) -> yields Slice<S>:
    assert k >= 1, "a chunk holds at least one element"
    var i = 0
    while i < self.len:
        yield piece(self, i, if i + k < self.len: i + k else: self.len)
        i += k

/// Adjacent pairs `(prev, x)` of `self`, left to right: `[10, 20, 30].adjacent` gives
/// `(10, 20)` then `(20, 30)`. Works on any `Seq` (`List<T>`, `[D]T`, `Str`, `Range`, views). \(O(1)\) per step.
/// A single element yields nothing too.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.adjacent.count() == 0`
/// order: left to right
/// traps: never
/// example: `[10, 20, 30].adjacent.collect() == [(10, 20), (20, 30)]`
@lint(adjacent_by_index, from: "self[v] < self[v.next!]", to: "self.adjacent", holes: "v")
@lint(adjacent_by_index, from: "self[v] <= self[v.next!]", to: "self.adjacent", holes: "v")
@lint(adjacent_by_index, from: "self[v] > self[v.next!]", to: "self.adjacent", holes: "v")
@lint(adjacent_by_index, from: "self[v] >= self[v.next!]", to: "self.adjacent", holes: "v")
@lint(adjacent_by_index, from: "self[v] == self[v.next!]", to: "self.adjacent", holes: "v")
@lint(adjacent_by_index, from: "self[v] != self[v.next!]", to: "self.adjacent", holes: "v")
@lint(adjacent_by_index, from: "i + 1 < n and self[i] < self[i + 1]", to: "self.adjacent", holes: "i, n")
@lint(adjacent_by_index, from: "i + 1 < n and self[i] <= self[i + 1]", to: "self.adjacent", holes: "i, n")
@lint(adjacent_by_index, from: "i + 1 < n and self[i] > self[i + 1]", to: "self.adjacent", holes: "i, n")
@lint(adjacent_by_index, from: "i + 1 < n and self[i] >= self[i + 1]", to: "self.adjacent", holes: "i, n")
@lint(adjacent_by_index, from: "i + 1 < n and self[i] == self[i + 1]", to: "self.adjacent", holes: "i, n")
@lint(adjacent_by_index, from: "i + 1 < n and self[i] != self[i + 1]", to: "self.adjacent", holes: "i, n")
@lint(adjacent_by_index, from: "let w = D(v.ord + 1)\nif self[v] < self[w]:\n    body", to: "self.adjacent", holes: "D, v, body")
@lint(adjacent_by_index, from: "let w = D(v.ord + 1)\nif self[v] <= self[w]:\n    body", to: "self.adjacent", holes: "D, v, body")
@lint(adjacent_by_index, from: "let w = D(v.ord + 1)\nif self[v] > self[w]:\n    body", to: "self.adjacent", holes: "D, v, body")
@lint(adjacent_by_index, from: "let w = D(v.ord + 1)\nif self[v] >= self[w]:\n    body", to: "self.adjacent", holes: "D, v, body")
@lint(adjacent_by_index, from: "let w = D(v.ord + 1)\nif self[v] == self[w]:\n    body", to: "self.adjacent", holes: "D, v, body")
@lint(adjacent_by_index, from: "let w = D(v.ord + 1)\nif self[v] != self[w]:\n    body", to: "self.adjacent", holes: "D, v, body")
@get pub fn adjacent<S: Seq<T>, T>(self: S) -> yields (T, T):
    var prev: ?T = None
    for x in self:
        if let p = prev: yield (p, x)
        prev = Some(x)

/// Every pair `(x, y)` with `x` before `y`: `[10, 20, 30].pairs()` gives `(10, 20)`, `(10, 30)`,
/// `(20, 30)`. \(O(n^2)\) to walk; for adjacent pairs use `adjacent`.
///
/// cost: \(O(n^2)\) to walk, \(O(1)\) per pair
/// empty: `let a: List<Int> = []; a.pairs().count() == 0`
/// order: by the first element's position, then the second's
/// traps: never
/// example: `[1, 2, 3].pairs().count() == 3`
/// see: `adjacent` for neighbours, `Pair<V>` for a table keyed by unordered pairs
@lint(manual_pair_loop, from: "for i in 0..n:\n    for j in i + 1..n:\n        if self[i] + self[j] == t:\n            c += 1", to: "c += self.pairs().count(p => p.0 + p.1 == t)", holes: "n, t, c")
@lint(manual_pair_loop, from: "for i in 0..self.len:\n    for j in i + 1..self.len:\n        if self[i] + self[j] == t:\n            c += 1", to: "c += self.pairs().count(p => p.0 + p.1 == t)", holes: "t, c")
pub fn pairs<S: Seq<T>, T>(self: S) -> yields (T, T):
    let xs: List<T> = List(self)
    for i in 0..xs.len:
        for j in i + 1..xs.len: yield (xs[i], xs[j])

/// Every element paired with the one before it, starting with `(None, first)`:
/// `[10, 20, 30].with_prev` gives `(None, 10)`, `(10, 20)`, `(20, 30)`. \(O(1)\) per step.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.with_prev.count() == 0`
/// order: left to right, starting with `(None, first)`
/// traps: never
/// example: `[10, 20].with_prev.collect() == [(None, 10), (Some(10), 20)]`
@get pub fn with_prev<S: Seq<T>, T>(self: S) -> yields (?T, T):
    var prev: ?T = None
    for x in self:
        yield (prev, x)
        prev = x

/// Every element with the one after it, the last with the first: the
/// sides of a polygon, the links of a ring. One pair `(x, x)` for a
/// single element, None for an empty sequence. Works on any `Seq`.
///
///     for (u, v) in poly.cyclic_pairs(): area += cross(u, v)
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.cyclic_pairs().count() == 0`
/// order: left to right, then `(last, first)`
/// traps: never
/// example: `[1, 2, 3].cyclic_pairs().collect() == [(1, 2), (2, 3), (3, 1)]`
pub fn cyclic_pairs<S: Seq<T>, T>(self: S) -> yields (T, T):
    var first_val: ?T = None
    var prev: ?T = None
    for x in self:
        if let p = prev: yield (p, x)
        else: first_val = x
        prev = x
    if let p = prev, let f = first_val:
        yield (p, f)

/// The pairs `(x, b.nth(i))` for the `i`th `x` that `a` walks, as many as the shorter sequence has.
/// `a` is any `Seq`, a stream too (`zip(xs.cyclic_pairs(), w)`); `b` is read by position. \(O(\min(\lvert a \rvert, \lvert b \rvert))\).
/// Either input empty yields nothing.
///
/// cost: \(O(\min(\lvert a \rvert, \lvert b \rvert))\)
/// empty: `let a: List<Int> = []; zip(a, "ab").count() == 0`
/// order: position by position, stopping at the shorter
/// traps: never
/// example: `zip([1, 2, 3], "ab").collect() == [(1, 'a'), (2, 'b')]`
@same_keys
pub fn zip<A: Seq<T>, B: Positional<U>, T, U>(a: A, b: B) -> Zipped<A, B, T, U> = Zipped(a, b)

/// The first and second halves of each pair, as two lists: the inverse of `zip`.
///
/// cost: \(O(n)\), two new lists
/// empty: `let a: List<(Int, Char)> = []; a.unzip() == ([], [])`
/// order: the order of `self`, keyed from 0
/// traps: never
/// example: `[(1, 'a'), (2, 'b')].unzip() == ([1, 2], ['a', 'b'])`
pub fn unzip<S: Seq<(A, B)>, A, B>(self: S) -> (List<A>, List<B>):
    var (xs, ys): (List<A>, List<B>) = ([], [])
    for (x, y) in self:
        xs.push(x)
        ys.push(y)
    (xs, ys)

/// The value `m` holds for `k`, after storing `v` there if it held none: `ids.get_or_insert(name, ids.len)`
/// gives each new name the next id. \(O(\log n)\).
///
/// cost: \(O(\log n)\)
/// traps: never
/// example: `var m: Map<Str, Int> = Map(); m.get_or_insert("a", 7) == 7 and m.get_or_insert("a", 9) == 7`
pub fn get_or_insert<K: Ord, V>(mut self: Map<K, V>, k: K, v: V) -> V:
    if not (k in self): self[k] = v
    self[k]

/// Replace the value under `k` by `f` of it, starting from `init` when `m` holds none:
/// `cnt.update(x, 0, c => c + 1)`. \(O(\log n)\).
///
/// cost: \(O(\log n)\), plus one call of `f`
/// traps: never
/// example: `var c: Map<Char, Int> = Map(); c.update('x', 0, v => v + 1); c['x'] == 1`
pub fn update<K: Ord, V>(mut self: Map<K, V>, k: K, init: V, f: V -> V):
    self[k] = f(self.get(k) ?? init)

/// Add every element of `o` to `self` in place: `self | o`, named as on `Bitset`.
///
/// cost: \(O(m \log(n + m))\) for `m` elements in `o`
/// empty: `var s: Set<Int> = Set(); s.insert(1); let t: Set<Int> = Set(); s.union(t); s.sorted() == [1]`
/// traps: never
/// example: `var s: Set<Int> = Set(); s.insert(1); var t: Set<Int> = Set(); t.insert(2); s.union(t); s.sorted() == [1, 2]`
pub fn union<T: Ord>(mut self: Set<T>, o: Set<T>):
    for x in o: self.insert(x)

/// Keep only the elements of `self` that are also in `o`, in place: `self & o`, named as on `Bitset`.
///
/// cost: \(O(n \log m)\) for `m` elements in `o`
/// empty: `var s: Set<Int> = Set(); s.insert(1); let t: Set<Int> = Set(); s.intersect(t); s.sorted() == []`
/// traps: never
/// example: `var s: Set<Int> = Set(); s.insert(1); s.insert(2); var t: Set<Int> = Set(); t.insert(2); s.intersect(t); s.sorted() == [2]`
pub fn intersect<T: Ord>(mut self: Set<T>, o: Set<T>):
    let gone = [x for x in self if not o.contains(x)]
    for x in gone: self.remove(x)

/// Take every element of `o` out of `self` in place: `self - o`, named as on `Bitset`.
///
/// cost: \(O(m \log n)\) for `m` elements in `o`
/// empty: `var s: Set<Int> = Set(); s.insert(1); let t: Set<Int> = Set(); s.subtract(t); s.sorted() == [1]`
/// traps: never
/// example: `var s: Set<Int> = Set(); s.insert(1); s.insert(2); var t: Set<Int> = Set(); t.insert(2); s.subtract(t); s.sorted() == [1]`
pub fn subtract<T: Ord>(mut self: Set<T>, o: Set<T>):
    for x in o: self.remove(x)

/// Rotate `self` left by `k` in place (or only the subrange `range` when given): what was at
/// offset `k % len` comes first.
///
/// Signed Euclidean `k`: negative `k`
/// rotates right. Works on any `IndexedMut` (`List<T>`, `[D]T`, `Str`). \(O(n)\), \(O(1)\) extra space.
///
/// cost: \(O(n)\), \(O(1)\) extra space
/// empty: `var a: List<Int> = []; a.rotate_left(3); a == []`
/// traps: when `range` falls outside the keys
/// example: `var a = [1, 2, 3, 4]; a.rotate_left(1); a == [2, 3, 4, 1]`
pub fn rotate_left<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = None):
    var lo = 0
    var hi = self.len
    if let r = range:
        if r.is_empty: return
        lo = r.first!
        hi = r.last! + 1
    let len = hi - lo
    if len <= 1: return
    let s = k % len
    if s == 0: return
    reverse(mut self, range: lo..lo + s)
    reverse(mut self, range: lo + s..hi)
    reverse(mut self, range: lo..hi)

/// Rotate `self` right by `k` in place (or only the subrange `range` when given). Works on any `IndexedMut`.
///
/// cost: \(O(n)\), \(O(1)\) extra space
/// empty: `var a: List<Int> = []; a.rotate_right(3); a == []`
/// traps: when `range` falls outside the keys
/// example: `var a = [1, 2, 3, 4]; a.rotate_right(1); a == [4, 1, 2, 3]`
pub fn rotate_right<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = None):
    rotate_left(mut self, -k, range:)

/// A new list rotated left by `k`: what was at offset `k % len` comes first.
/// Signed Euclidean `k`: negative `k` rotates right. Works on any `Seq`. \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.rotated_left(3) == [] and a.rotated_left(-2) == []`
/// order: a new `List`, keyed from 0
/// traps: never
/// example: `[1, 2, 3].rotated_left(-1) == [3, 1, 2]`
pub fn rotated_left<S: Seq<T>, T>(self: S, k: Int) -> List<T>:
    let n = self.len
    if n <= 1: return self.collect()
    let s = k % n
    if s == 0: return self.collect()
    var out: List<T> = []
    out.reserve(n)
    var head: List<T> = []
    head.reserve(s)
    var idx = 0
    for x in self:
        if idx < s: head.push(x)
        else: out.push(x)
        idx += 1
    for x in head: out.push(x)
    out

/// A new list rotated right by `k`. Works on any `Seq`.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.rotated_right(3) == [] and a.rotated_right(-2) == []`
/// order: a new `List`, keyed from 0
/// traps: never
/// example: `[1, 2, 3].rotated_right(1) == [3, 1, 2]`
pub fn rotated_right<S: Seq<T>, T>(self: S, k: Int) -> List<T>:
    rotated_left(self, -k)

/// Step `self` to the next arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted ascending, when it was the last one. Works on any `IndexedMut`. \(O(n)\), amortized \(O(1)\).
///
/// Equal elements are one arrangement, so `[1, 1, 2]` has three.
///
/// cost: \(O(n)\) worst case, amortized \(O(1)\) over all arrangements
/// empty: `var a: List<Int> = []; not a.next_permutation() and a == []`
/// ties: `var a = [1, 2, 1]; a.next_permutation() and a == [2, 1, 1]`
/// traps: never
/// example: `var a = [1, 2, 3]; a.next_permutation() and a == [1, 3, 2]`
pub fn next_permutation<S: IndexedMut<D, T>, D: Domain, T: Ord>(mut self: S) -> Bool:
    var i = self.len - 2
    while i >= 0 and not (self.nth(i) < self.nth(i + 1)): i -= 1
    if i < 0:
        reverse(mut self)
        return false
    var j = self.len - 1
    while not (self.nth(i) < self.nth(j)): j -= 1
    self.swap_idx(i, j)
    var (l, r) = (i + 1, self.len - 1)
    while l < r:
        self.swap_idx(l, r)
        l += 1
        r -= 1
    true

/// Step `self` to the previous arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted descending, when it was the first (ascending) one. Works on any `IndexedMut`. \(O(n)\), amortized \(O(1)\).
///
/// Equal elements are one arrangement.
///
/// cost: \(O(n)\) worst case, amortized \(O(1)\) over all arrangements
/// empty: `var a: List<Int> = []; not a.prev_permutation() and a == []`
/// ties: `var a = [2, 1, 1]; a.prev_permutation() and a == [1, 2, 1]`
/// traps: never
/// example: `var a = [1, 3, 2]; a.prev_permutation() and a == [1, 2, 3]`
pub fn prev_permutation<S: IndexedMut<D, T>, D: Domain, T: Ord>(mut self: S) -> Bool:
    var i = self.len - 2
    while i >= 0 and not (self.nth(i + 1) < self.nth(i)): i -= 1
    if i < 0:
        reverse(mut self)
        return false
    var j = self.len - 1
    while not (self.nth(j) < self.nth(i)): j -= 1
    self.swap_idx(i, j)
    var (l, r) = (i + 1, self.len - 1)
    while l < r:
        self.swap_idx(l, r)
        l += 1
        r -= 1
    true

/// The stretches of equal neighbours (or equal `by:` keys), in order, each a slice of `self`.
///
/// `"aabccc".runs()` walks `"aa"`, `"b"`, `"ccc"`, and
/// `[r.len for r in s.runs()]` is `[2, 1, 3]`. Each run's `.keys` is its range and
/// `r[r.keys.first!]` its key. With `link:`, a run is a maximal piece where every adjacent
/// pair `(x, y)` satisfies `link(x, y)`, and `by:` is ignored: `runs(by: key)` is
/// `runs(link: (x, y) => key(x) == key(y))`. \(O(1)\) to make, \(O(n)\) to walk.
///
/// cost: \(O(1)\) to make, \(O(n)\) to walk
/// empty: `let a: List<Int> = []; a.runs().count() == 0`
/// order: left to right; each run is a slice keeping the keys of `self`
/// traps: never
/// example: `[r.len for r in "aabccc".runs()] == [2, 1, 3]`
/// example: `[r.len for r in [1, 2, 4, 5, 6].runs(link: (x, y) => y == x + 1)] == [2, 3]`
pub fn runs<S: Indexed<D, T>, D: Domain, T, K: Eq>(
    self: S,
    by: T -> K = x => x,
    link: ?((T, T) -> Bool) = None,
) -> yields Slice<S>:
    let n = self.len
    var i = 0
    if let Some(linked) = link:
        while i < n:
            var j = i + 1
            while j < n and linked(self.nth(j - 1), self.nth(j)): j += 1
            yield piece(self, i, j)
            i = j
    elif n > 0:
        var k = by(self.nth(0))
        while i < n:
            var j = i + 1
            while j < n:
                let next = by(self.nth(j))
                if not (next == k):
                    k = next
                    break
                j += 1
            yield self[self.keys.nth(i)..=self.keys.nth(j - 1)]
            i = j

/// Bucket the keys of `self` by value: `b[k]` lists every `i` with `self[i] == k`, in key order.
///
/// The result is keyed by the value domain, so group by any domain-valued key with
/// `a.map(key).buckets()`.
/// \(O(\lvert D \rvert + \lvert K \rvert)\) time and memory.
///
/// cost: \(O(\lvert D \rvert + \lvert K \rvert)\) time and memory
/// empty: `domain D = 0..0; domain K = 0..2; let a: [D]K = []; a.buckets() == [[], []]`
/// order: each bucket lists keys of `self` in increasing order; the result is keyed by `K`
/// traps: never
/// example: `domain D = 0..3; domain K = 0..2; let a: [D]K = [1, 0, 1]; a.buckets() == [[1], [0, 2]]`
pub fn buckets<D, K: Domain>(self: [D]K) -> [K]List<D>:
    var out: [K]List<D> = []
    for i in D: out[self[i]].push(i)
    out

/// The distinct values of a coordinate compression, in increasing order, indexed by their
/// own domain `D`. `xs[i]` is the value at index `i`, `xs.index_of(val)` the index of a value that
/// was compressed, and `xs.first_ge(val)` / `xs.last_le(val)` the nearest index (`?D`) for any value.
pub struct Compressed<D, T>(vals: [D]T):
    /// \(O(1)\). The value at index `i`; `slice` takes a `Range<D>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let (domain C, xs) = compress([30, 10]); xs.get(C.first!) == 10`
    pub fn get(self, i: D) -> T = vals[i]

    /// The compressed values over a range `r` of `xs.D`, as a list.
    ///
    /// cost: \(O(r.\mathit{len})\)
    /// empty: `let (domain C, xs) = compress([3, 1, 2]); xs.slice(C.first!..C.first!) == []`
    /// order: increasing values; a new `List`, keyed from 0
    /// traps: never
    /// example: `let (domain C, xs) = compress([3, 1, 2]); xs.slice(C.all) == [1, 2, 3]`
    pub fn slice(self, r: Range<D>) -> List<T> = [vals[i] for i in r]

    /// How many distinct values there are.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `compress([5, 5, 7]).1.len == 2`
    @get pub fn len(self) -> Int = vals.len

    /// Whether no value was compressed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let e: List<Int> = []; compress(e).1.is_empty and not compress([4]).1.is_empty`
    @get pub fn is_empty(self) -> Bool = vals.len == 0

    /// \(O(\mathit{len})\). Visit every compressed index with its value.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `let e: List<Int> = []; let (domain C, xs) = compress(e); xs.items.collect() == []`
    /// order: increasing index, so increasing value
    /// traps: never
    /// example: `let (domain C, xs) = compress([30, 10, 30]); [v for (_, v) in xs.items] == [10, 30]`
    @get pub fn items(self) -> yields (D, T):
        for i in D: yield (i, vals[i])

    /// The index of `val`, which must be one of the compressed values.
    /// Ties can't arise: the compressed values are distinct.
    ///
    /// cost: \(O(\log n)\)
    /// ties: `let (domain C, xs) = compress([30, 10, 30]); xs.index_of(30) == C.last!`
    /// traps: when `val` was not compressed
    /// example: `let (domain C, xs) = compress([30, 10]); xs.index_of(30) == C.last!`
    /// trap: `let (domain C, xs) = compress([30, 10]); xs.index_of(20)`
    pub fn index_of(self, val: T) -> D:
        let i = vals.first_ge(val)
        if let k = i, vals[k] == val: return k
        assert false, "`index_of` is for a value that was compressed"
        i!

    /// The index of `val` if it was compressed, or `None`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: never
    /// example: `compress([30, 10]).1.try_index_of(20) == None`
    pub fn try_index_of(self, val: T) -> ?D:
        if let i = vals.first_ge(val), vals[i] == val: i else: None

    /// Whether `val` is one of the compressed values.
    ///
    /// cost: \(O(\log n)\)
    /// traps: never
    /// example: `compress([30, 10]).1.contains(10) == true`
    pub fn contains(self, val: T) -> Bool = self.try_index_of(val) != None

    /// The first index whose value is `>= val`, or `None`.
    /// Ties can't arise: the compressed values are distinct.
    ///
    /// cost: \(O(\log n)\)
    /// ties: `let (domain C, xs) = compress([10, 30, 30]); xs.first_ge(30) == Some(C.last!)`
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.first_ge(20) == Some(C.last!)`
    pub fn first_ge(self, val: T) -> ?D = vals.first_ge(val)

    /// The first index whose value is `> val`, or `None`.
    /// Ties can't arise: the compressed values are distinct.
    ///
    /// cost: \(O(\log n)\)
    /// ties: `let (domain C, xs) = compress([10, 10, 30]); xs.first_gt(10) == Some(C.last!)`
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.first_gt(30) == None`
    pub fn first_gt(self, val: T) -> ?D = vals.first_gt(val)

    /// The last index whose value is `<= val`, or `None`.
    /// Ties can't arise: the compressed values are distinct.
    ///
    /// cost: \(O(\log n)\)
    /// ties: `let (domain C, xs) = compress([10, 30, 30]); xs.last_le(30) == Some(C.last!)`
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.last_le(20) == Some(C.first!)`
    pub fn last_le(self, val: T) -> ?D = vals.last_le(val)

    /// The last index whose value is `< val`, or `None`.
    /// Ties can't arise: the compressed values are distinct.
    ///
    /// cost: \(O(\log n)\)
    /// ties: `let (domain C, xs) = compress([10, 10, 30]); xs.last_lt(30) == Some(C.first!)`
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.last_lt(10) == None`
    pub fn last_lt(self, val: T) -> ?D = vals.last_lt(val)

/// Coordinate compression: `let (domain C, xs) = compress(all)` sorts the values, drops the
/// duplicates and binds a fresh domain `C` for them.
///
/// So an array over them is `[C]Int` and an
/// index of one compression cannot be used in another. Works on any `Seq`. \(O(n \log n)\).
///
/// cost: \(O(n \log n)\)
/// empty: `let e: List<Int> = []; let (domain C, xs) = compress(e); C.count == 0 and xs.is_empty`
/// order: increasing values, indexed by the fresh domain `C`
/// traps: never
/// example: `let (domain C, xs) = compress([30, 10, 30]); xs.len == 2 and xs.index_of(30) == C.last!`
pub fn compress<S: Seq, T: Ord>(self: S) -> (domain C, Compressed<C, T>):
    var raw: List<T> = self.collect()
    raw.sort()
    var uniq: List<T> = []
    for x in raw:
        if uniq.is_empty or uniq.last != x: uniq.push(x)
    domain C = 0..uniq.len
    let vals = [C] i => uniq[i.ord]
    (C, Compressed(vals))

/// Split a range at cuts: `let (domain P, piece) = D.pieces(cuts: cs)` binds a fresh domain of the
/// non-empty pieces between consecutive cuts, with `piece[p]` the `Range<D>` that piece covers.
///
/// The range's own ends always count and repeated cuts drop; pieces come in order. It is coordinate compression for
/// pieces: a segment tree over `P` stores `piece[p].len`, and no +1/-1 converts back.
/// \(O(k \log k)\) for k cuts.
///
/// cost: \(O(k \log k)\) for k cuts
/// empty: `let (domain P, piece) = (0..0).pieces(cuts: [0]); P.count == 0`
/// order: pieces left to right; `piece` is keyed by the fresh domain `P`
/// traps: when a cut lies outside the range
/// example: `let (domain P, piece) = (0..10).pieces(cuts: [3, 7]); [piece[p].len for p in P] == [3, 4, 3]`
pub fn pieces<D: Ord, S: Seq>(self: Range<D>, cuts: S) -> (domain P, [P]Range<D>):
    let a = self.l
    let b = self.r
    if self.is_empty:
        for c in cuts: assert a <= c <= b, "pieces: cut outside range"
        domain P = 0..0
        return (P, [P] _ => span(a, a))
    var pts: List<Cut<D>> = [a, b]
    for c in cuts:
        assert a <= c <= b, "pieces: cut outside range"
        pts.push(c)
    pts.sort()
    var uniq: List<Cut<D>> = []
    for x in pts:
        if uniq.is_empty or uniq.last != x: uniq.push(x)
    let m = uniq.len - 1
    domain P = 0..m
    let ps = [P] i => span(uniq[i.ord], uniq[i.ord + 1])
    (P, ps)

/// The range of pieces exactly covering `r`, whose ends must be among the cuts the pieces were
/// made from (traps otherwise). `t.apply(piece.covering(a..=b), x)`. \(O(\log k)\).
///
/// cost: \(O(\log k)\)
/// empty: `let (domain P, piece) = (0..10).pieces(cuts: [3, 7]); piece.covering(3..3).len == 0`
/// traps: when an end of `r` is not one of the cuts
/// order: pieces left to right, keyed by `P`
/// example: `let (domain P, piece) = (0..10).pieces(cuts: [3, 7]); piece.covering(3..10).len == 2`
/// trap: `let e: List<Int> = []; let (domain P, piece) = (0..0).pieces(cuts: e); piece.covering(0..1)`
pub fn covering<P, D>(self: [P]Range<D>, r: Range<D>) -> Range<P>:
    if P.count == 0:
        assert r.is_empty, "covering: an end of the range is not a cut"
        return P.range
    let i = (0..P.count).first_true(m => self[P(m)].l >= r.l) ?? P.count
    let j = (0..P.count).first_true(m => self[P(m)].r > r.r) ?? P.count
    let at_i = if i < P.count: self[P(i)].l else: self[P.last!].r
    let at_j = if j > 0: self[P(j - 1)].r else: self[P.first!].l
    assert at_i == r.l and at_j == r.r, "covering: an end of the range is not a cut"
    span(P.cuts(i), P.cuts(j))

/// The first `f(x)` that is not `None`, stopping there; `None` if every one is.
/// \(O(1)\) per element walked.
///
/// cost: \(O(n)\), \(O(1)\) per element walked, stopping at the first `Some`
/// empty: `let a: List<Int> = []; a.first_some(x => Some(x)) == None`
/// ties: `[1, 2].first_some(x => Some(x * 10)) == Some(10)`
/// traps: never
/// example: `[1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None) == Some(40)`
pub fn first_some<S: Seq<T>, T, U>(self: S, f: T -> ?U) -> ?U:
    for x in self:
        let y = f(x)
        if y.is_some: return y
    None

/// The first element satisfying `p`, `None` if None does; stops there. `(1..=m).find(ok)` is the
/// least `ok` integer. \(O(k)\) for a first match at position \(k\).
///
/// cost: \(O(k)\) calls of `p` for a first match at position \(k\)
/// empty: `let a: List<Int> = []; a.find(x => x > 0) == None`
/// ties: `[(1, 'a'), (1, 'b')].find(p => p.0 == 1) == Some((1, 'a'))`
/// traps: never
/// example: `(1..=10).find(x => x * x > 20) == Some(5)`
@lint(count_until, from: "var x = 1\nvar k = 0\nwhile x < n:\n    x *= c\n    k += 1", to: "let k = (0..).find(k => c ** k >= n)!", holes: "c, n")
@lint(count_until, from: "var x = 1\nvar k = 0\nwhile x < n:\n    k += 1\n    x *= c", to: "let k = (0..).find(k => c ** k >= n)!", holes: "c, n")
@lint(count_until, from: "var x = 1\nvar k = 0\nwhile x <= n:\n    x *= c\n    k += 1", to: "let k = (0..).find(k => c ** k > n)!", holes: "c, n")
@lint(count_until, from: "var x = 1\nvar k = 0\nwhile x <= n:\n    k += 1\n    x *= c", to: "let k = (0..).find(k => c ** k > n)!", holes: "c, n")
@lint(manual_mex, from: "while seen[m]:\n    m += 1", to: "(lo..=hi).find(m => not seen[m])", holes: "seen, m")
pub fn find<S: Seq<T>, T>(self: S, p: T -> Bool) -> ?T:
    for x in self:
        if p(x): return Some(x)
    None

/// Gives the items of a list a fresh domain, so arrays over them are index-safe:
/// `let (domain Arc, arcs) = [...].with_domain()`. Moves the list, \(O(1)\).
///
/// cost: \(O(1)\); moves the list
/// empty: `let e: List<Int> = []; let (domain I, xs) = e.with_domain(); I.count == 0`
/// order: the list's order, keyed by the fresh domain
/// traps: never
/// example: `let (domain I, xs) = [7, 8].with_domain(); xs[I.first!] == 7`
pub fn with_domain<T>(self: List<T>) -> (domain I, [I]T):
    domain I = 0..self.len
    (I, [I] i => self[i.ord])

/// The cycle of a permutation through `from`: `from`, `perm[from]`, ... until it
/// returns to `from`. \(O(1)\) to make, \(O(\text{cycle length})\) to walk.
/// Never empty: it yields at least `from`.
///
/// cost: \(O(1)\) to make, \(O(1)\) per step
/// order: `from`, `self[from]`, ... stopping before `from` comes back
/// traps: never; a non-permutation that never returns to `from` walks forever
/// example: `[1, 2, 0].cycle(from: 0).collect() == [0, 1, 2]`
/// empty: `[0].cycle(from: 0).collect() == [0]`
pub fn cycle<D>(self: [D]D, from: D) -> yields D:
    var x = from
    while true:
        yield x
        x = self[x]
        if x == from: return

// ---- Keyed sorted lookup, ordered Map 4-lookups, and List/Array helpers

/// First index in sorted `self` with `by(self[i]) > target`, or `None`.
///
/// cost: \(O(\log n)\)
/// empty: `let a: List<Int> = []; a.first_gt(3) == None`
/// ties: `[3, 3, 4].first_gt(3) == Some(2)`
/// traps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;
/// otherwise unsorted input gives an unspecified key
/// example: `[1, 3, 3, 5].first_gt(3) == Some(3)`
pub fn first_gt<D, T, K: Ord>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    let lo = (0..D.count).first_true(m => by(self[D.nth(m)]) > target) ?? D.count
    if lo < D.count: Some(D.nth(lo)) else: None

/// Last index in sorted `self` with `by(self[i]) < target`, or `None`.
///
/// cost: \(O(\log n)\)
/// empty: `let a: List<Int> = []; a.last_lt(3) == None`
/// ties: `[2, 3, 3].last_lt(3) == Some(0)`
/// traps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;
/// otherwise unsorted input gives an unspecified key
/// example: `[1, 3, 3, 5].last_lt(3) == Some(0)`
pub fn last_lt<D, T, K: Ord>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    let lo = (0..D.count).first_true(m => by(self[D.nth(m)]) >= target) ?? D.count
    if lo > 0: Some(D.nth(lo - 1)) else: None

// ---- Permutations, monotone stack, and sliding-window extrema

/// Inverse of bijection `self: [I]V`: `inv[self[i]] == i` for all `i in I`.
/// Traps if `self` is not a bijection. \(O(\lvert I \rvert)\).
///
/// cost: \(O(\lvert I \rvert)\)
/// empty: `domain D = 0..0; let p: [D]D = []; p.inverse() == []`
/// order: keyed by `V`
/// traps: when `self` is not a bijection
/// example: `[2, 0, 1].inverse() == [1, 2, 0]`
pub fn inverse<I, V>(self: [I]V) -> [V]I:
    assert I.count == V.count, "inverse requires domains of equal size"
    var inv: [V]?I = None
    for i in I:
        let v = self[i]
        assert inv[v] == None, "inverse requires a bijection (duplicate image)"
        inv[v] = i
    [V] v => inv[v]!

/// Function composition: `compose(f, g)[a] == g[f[a]]`.
///
/// cost: \(O(\lvert A \rvert)\)
/// empty: `domain D = 0..0; let f: [D]D = []; let g: [D]Int = []; compose(f, g) == []`
/// order: keyed by `A`
/// traps: never
/// example: `domain D = 0..2; let f: [D]D = [1, 0]; let g: [D]Int = [5, 6]; compose(f, g) == [6, 5]`
pub fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C = [A] a => g[f[a]]

/// Disjoint cycle decomposition of permutation `p`, each cycle starting at its smallest element.
///
/// cost: \(O(\lvert V \rvert)\)
/// empty: `domain D = 0..0; let p: [D]D = []; perm_cycles(p) == []`
/// order: cycles by smallest element; each starts there and follows `p`
/// traps: when `p` is not a permutation
/// example: `perm_cycles([1, 0, 2]) == [[0, 1], [2]]`
/// trap: `perm_cycles([0, 0, 1])`
pub fn perm_cycles<V>(self: [V]V) -> List<List<V>>:
    var vis: [V]Bool = false
    var out: List<List<V>> = []
    for start in V:
        if vis[start]: continue
        var cyc: List<V> = []
        var u = start
        while not vis[u]:
            vis[u] = true
            cyc.push(u)
            u = self[u]
        assert u == start, "not a permutation: {u} is reached twice"
        out.push(cyc)
    out

/// `k`-th iterate of permutation `p` (`k >= 0`), computed via cycle decomposition.
///
/// cost: \(O(\lvert V \rvert)\)
/// empty: `domain D = 0..0; let p: [D]D = []; perm_pow(p, 5) == []`
/// order: keyed by `V`
/// traps: when `k < 0` or `p` is not a permutation
/// example: `perm_pow([1, 2, 0], 2) == [2, 0, 1]`
pub fn perm_pow<V>(p: [V]V, k: 0..) -> [V]V:
    assert k >= 0, "perm_pow requires k >= 0"
    var out = [V] v => v
    for cyc in perm_cycles(p):
        let m = cyc.len
        let shift = k % m
        for j in 0..m:
            out[cyc[j]] = cyc[(j + shift) % m]
    out

/// Number of inversions: pairs `(i, j)` with `i < j` and `a[i] > a[j]`. Works on any `Seq`.
///
/// cost: \(O(n \log n)\)
/// empty: `let a: List<Int> = []; a.inversions() == 0`
/// ties: `[2, 2, 1].inversions() == 2`
/// traps: never
/// example: `[3, 1, 2].inversions() == 2`
/// example: `inversions([3, 1, 2]) == 2`
@lint(inversions_by_fenwick, from: "var bit = Fenwick<C, Int>()\nvar inv = 0\nfor i in I:\n    let id = comp.index_of(self[i])\n    inv += i.ord - bit.prefix(id.r)\n    bit.add(id, 1)", to: "let inv = self.inversions()", holes: "C, I, comp")
pub fn inversions<S: Seq<T>, T: Ord>(self: S) -> Int:
    var cur: List<T> = self.collect()
    let n = cur.len
    var tmp = cur.clone()
    var total = 0
    var w = 1
    while w < n:
        var lo = 0
        while lo < n:
            let mid = if lo + w < n: lo + w else: n
            let hi = if lo + 2 * w < n: lo + 2 * w else: n
            var (i, j, k) = (lo, mid, lo)
            while i < mid or j < hi:
                if j == hi or (i < mid and not (cur[j] < cur[i])):
                    tmp[k] = cur[i]
                    i += 1
                else:
                    total += mid - i
                    tmp[k] = cur[j]
                    j += 1
                k += 1
            lo = hi
        (cur, tmp) = (tmp, cur)
        w *= 2
    total

/// Swap two positions of a domain array in place; safely a no-op when `i == j`.
/// An empty array has no key, so no call is possible.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `var a = [1, 2]; a.swap_at(0, 1); a == [2, 1]`
/// empty: `domain D = 0..0; var a: [D]Int = []; a.swap_at(D.first!, D.first!); a == []` (traps R0301)
pub fn swap_at<D, T>(mut self: [D]T, i: D, j: D):
    if i == j: return
    let t = self[i]
    self[i] = self[j]
    self[j] = t

/// Swap positions `i` and `j` of any `IndexedMut`, counted from 0, in place; a no-op when
/// `i == j`.
///
/// cost: \(O(1)\)
/// traps: when `i` or `j` is not in `0..self.len`
/// example: `var a = [1, 2, 3]; a.swap_idx(0, 2); a == [3, 2, 1]`
/// empty: `var a: List<Int> = []; a.swap_idx(0, 0); a == []` (traps R0101)
pub fn swap_idx<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, i: Int, j: Int):
    let (a, b) = (self.keys.nth(i), self.keys.nth(j))
    if i == j: return
    let t = self[a]
    self[a] = self[b]
    self[b] = t

/// `self` cut or padded with `fill` to length `n`: `a.resized(size, Z(0))` instead of
/// `[if i < a.len: a[i] else: Z(0) for i in 0..size]`. Works on any `Indexed` sequence. \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.resized(2, 7) == [7, 7]`
/// order: a new `List`, keyed from 0
/// traps: when `n < 0` or `n >= 2^31`
/// example: `[1, 2, 3].resized(5, 0) == [1, 2, 3, 0, 0]`
pub fn resized<S: Indexed<D, T>, D: Domain, T>(self: S, n: Len, fill: T) -> List<T> = [
    if i < self.len: self.nth(i) else: fill for i in 0..n
]

/// \(O(1)\). Pop the last element if `ok` holds for it, else leave `self` alone and give `None`.
/// A monotonic stack is `while let top = st.pop_if(t => t.h >= h): ...`.
///
/// cost: \(O(1)\)
/// empty: `var s: List<Int> = []; s.pop_if(x => x > 3) == None`
/// traps: never
/// example: `var s = [1, 5]; s.pop_if(x => x > 3) == Some(5)`
pub fn pop_if<T>(
    mut self: List<T>,
    ok: T -> Bool,
) -> ?T = if self.len > 0 and ok(self.last!): self.pop() else: None

/// \(O(\mathit{popped})\). Pop from the back while `ok` holds for the last element; the popped
/// elements, last first. The back twin of `Deque.pop_front_while`.
///
/// cost: \(O(\mathit{popped})\)
/// empty: `var s: List<Int> = []; s.pop_while(x => x > 3) == []`
/// order: last element first
/// traps: never
/// example: `var s = [1, 4, 5]; s.pop_while(x => x > 3) == [5, 4] and s == [1]`
pub fn pop_while<T>(mut self: List<T>, ok: T -> Bool) -> List<T>:
    var out: List<T> = []
    while self.len > 0 and ok(self.last!): out.push(self.pop()!)
    out

/// \(O(n)\), allocates. A separate copy: `let b = a` moves, `let b = a.clone()` copies.
///
/// cost: \(O(n)\) for a value of size \(n\), allocates for heap values
/// traps: never
/// example: `let a = [1]; var b = a.clone(); b.push(2); a == [1]`
pub fn clone<T>(self: T) -> T = copy(self)

/// Move all elements out of `self` in \(O(1)\), leaving it empty.
///
/// cost: \(O(1)\)
/// empty: `var a: List<Int> = []; a.take_all() == [] and a == []`
/// order: unchanged; `self` is left empty
/// traps: never
/// example: `var a = [1, 2]; a.take_all() == [1, 2] and a == []`
pub fn take_all<T>(mut self: List<T>) -> List<T> = self.take()

// =============================================================================
// Spec surface (`= todo` where not yet implemented)
// =============================================================================

// --- Lazy views, slicing, and stream / combinator generators (W4, W6) ---

/// A sequence: `for x in s` walks it in order, and each `for` runs it from the start
/// (R07.3.1). Generators, adapters, collections and ranges implement it (decision 509).
/// It fills the iteration role (R11.6.1): `for` walks a value through `each`, which a type may
/// write itself and otherwise walks `iter`.
@role(iteration)
pub trait Seq<T>:
    /// Walks the sequence from its start.
    @lint(index_only_loop, from: "for i in 0..self.len:\n    b", to: "for x in self:\n    b", reads: "self[i] -> x", each: "x in self", holes: "b")
    @lint(index_only_loop, from: "for i in self.indices:\n    b", to: "for x in self:\n    b", reads: "self[i] -> x", each: "x in self", holes: "b")
    fn iter(self) -> yields T
    /// The sequence's elements from its start: `iter`, unless the type writes its own walk.
    ///
    /// cost: what `iter` costs
    /// empty: `Heap<Int>().each().collect() == []`
    /// traps: never
    /// example: `Heap([3, 1, 2]).each().collect() == [1, 2, 3]`
    fn each(self) -> yields T:
        for x in self.iter(): yield x

/// A `-> yields T` call made outside a `for`: the call, held. Nothing runs until something walks
/// it, and each walk reruns it from the start (R07.6.2). It walks and nothing more: keep the
/// results with `List(g)`.
@role(recipe) pub struct Gen<T>:
    run: () -> ()

impl Seq<T> for Gen<T>:
    fn iter(self) -> yields T:
        for x in self: yield x

/// A `Seq` whose elements carry keys: `s.items` walks `(key, x)`, and keys survive `filter`
/// (R07.3.2).
pub trait Keyed<D, T>: Seq<T>:
    /// Walks `(key, x)` in walk order.
    fn items(self) -> yields (D, T)

/// A sequence with an \(O(1)\) length (R07.3.3). `domain V = xs` has `xs.len` positions
/// (R04.2.29).
@role(counted)
pub trait Sized:
    /// The element count.
    @get fn len(self) -> Int

/// The element count of any `Sized` value, as a call: `len(xs)` is `xs.len`.
///
/// cost: what `s.len` costs, \(O(1)\) for every core sequence
/// empty: `let a: List<Int> = []; len(a) == 0`
/// traps: never
/// example: `len([b for b in [1, 2, 3] if b > 1]) == 2`
pub fn len<S: Sized>(s: S) -> Int = s.len

/// A `Seq` that can also walk from its end, which is what `.rev` needs.
pub trait Bidir<T>: Seq<T>:
    /// Walks the sequence from its end.
    fn iter_rev(self) -> yields T


impl Seq<T> for List<T>:
    fn iter(self) -> yields T:
        for x in self: yield x

impl Bidir<T> for List<T>:
    fn iter_rev(self) -> yields T:
        var i = self.len - 1
        while i >= 0:
            yield self[i]
            i -= 1

impl Sized for List<T>:
    @get fn len(self) -> Int = self.len

impl Keyed<Int, T> for List<T>:
    fn items(self) -> yields (Int, T):
        for i in 0..self.len: yield (i, self[i])

impl Seq<D> for Range<D>:
    fn iter(self) -> yields D:
        for x in self: yield x

impl Bidir<D> for Range<D>:
    fn iter_rev(self) -> yields D:
        for x in self.rev: yield x

impl Sized for Range<D>:
    @get fn len(self) -> Int = self.len

impl Seq<Char> for Str:
    fn iter(self) -> yields Char:
        for c in self: yield c

impl Bidir<Char> for Str:
    fn iter_rev(self) -> yields Char:
        var i = self.len - 1
        while i >= 0:
            yield self[i]
            i -= 1

impl Sized for Str:
    @get fn len(self) -> Int = self.len

impl Keyed<Int, Char> for Str:
    fn items(self) -> yields (Int, Char):
        for i in 0..self.len: yield (i, self[i])

impl Seq<T> for [D]T:
    fn iter(self) -> yields T:
        for x in self: yield x

impl Bidir<T> for [D]T:
    fn iter_rev(self) -> yields T:
        for v in D.rev: yield self[v]

impl Sized for [D]T:
    @get fn len(self) -> Int = self.len

impl<D: Domain> Sized for D:
    @get fn len(self) -> Int = D.count

/// A `Prefix` walks its running folds: the fold through each position (from the right for a
/// suffix table), in key order, so
/// `a.scan(+).min()` is the least running total and `inversions(a.scan(+))` counts its
/// inversions.
impl<D: Ordered, T> Seq<T> for Prefix<D, T>:
    fn iter(self) -> yields T:
        for v in D:
            yield self.through(v)

impl<D: Ordered, T> Keyed<D, T> for Prefix<D, T>:
    fn items(self) -> yields (D, T):
        for v in D:
            yield (v, self.through(v))

impl<D: Ordered, T> Bidir<T> for Prefix<D, T>:
    fn iter_rev(self) -> yields T:
        for v in D.rev:
            yield self.through(v)

/// A sized sequence read in \(O(1)\) by walk position: `s.nth(k)` is the element `k` steps into
/// the walk. Every `Indexed` is one, and so is the `.rev` of one, which is what `zip`'s later
/// arguments need (R07.6.4).
pub trait Positional<T>: Seq<T> + Sized:
    /// The element `k` steps into the walk, 0-based; traps unless `0 <= k < len`.
    fn nth(self, k: Int) -> T

/// `s[k]` read: the element at key `k`. A `List`, an array and a `Map` have it; a `Map`'s keys
/// are not positions, so it is not `Indexed` (R12.4.5).
@operator("[]")
pub trait IndexGet<K, T>:
    /// The element at key `k`; traps when `s` holds none there.
    fn at(self, k: K) -> T

/// `s[k]` as a place: a write through it lands in `s`, in place (R12.4.6). A `Map`'s starts an
/// absent key at the default of its values, so `m[k] += 1` counts (R07.12.9).
pub trait IndexMut<K, T>: IndexGet<K, T>:
    fn at(mut self, k: K) -> lends mut T
/// An `Indexed` sequence whose cells are places: what `sort`, `reverse` and `swap` write
/// (R12.4.6).
pub trait IndexedMut<K, T>: Indexed<K, T> + IndexMut<K, T>
/// `s[k] = v` as a write of `v` at `k`, not through a place: a `Map` inserts or replaces, and
/// its values need no default (spec 12, W8).
@operator("[]=")
pub trait IndexSet<K, T>:
    fn set(mut self, k: K, v: T)

/// A keyed, sized, two-way sequence read in \(O(1)\) at any key; `take`/`drop` give a slice.
pub trait Indexed<D, T>: IndexGet<D, T> + Keyed<D, T> + Positional<T> + Bidir<T>:
    type Taken = Slice<Self>
    /// The keys, in walk order.
    @get fn keys(self) -> Range<D>
    /// `s.cut_after(i)`: the cut after key `i` in walk order, `i.r` on a forward view; traps R0101
    /// unless `i` is a key of `s` (R07.4.14).
    ///
    /// cost: \(O(1)\)
    /// example: `domain V = 1..=4; let a = [V] v => Int(v); a.cut_after(V(2)) == V(2).r`
    fn cut_after(self, i: D) -> Cut<D> where D: Ordered:
        check_key(self.keys.contains(i))
        i.r
    /// `s.cut_before(i)`: the cut before key `i` in walk order, `i.l` on a forward view; traps R0101
    /// unless `i` is a key of `s` (R07.4.14).
    ///
    /// cost: \(O(1)\)
    /// example: `domain V = 1..=4; let a = [V] v => Int(v); a.cut_before(V(2)) == V(2).l`
    fn cut_before(self, i: D) -> Cut<D> where D: Ordered:
        check_key(self.keys.contains(i))
        i.l

/// \(O(1)\) to make. The view `s.filter(p)`: walks `s`, skipping elements that fail `p`, with
/// `s`'s keys. A lent value (R07.1.5).
pub struct Filtered<S, T>(
    /// \(O(1)\). The source sequence this view filters.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let a = [1, 2]; filter(a, x => x > 1).src == a`
    pub src: S,
    p: T -> Bool,
)

impl Seq<T> for Filtered<S: Seq<T>, T>:
    fn iter(self) -> yields T:
        for x in src:
            if p(x): yield x

impl Keyed<D, T> for Filtered<S: Keyed<D, T>, T>:
    fn items(self) -> yields (D, T):
        for (k, x) in src.items:
            if p(x): yield (k, x)

impl Bidir<T> for Filtered<S: Bidir<T>, T>:
    fn iter_rev(self) -> yields T:
        for x in src.rev:
            if p(x): yield x

/// \(O(1)\) to make. The view `s.map(f)`: `f` of each element, with `s`'s keys; Indexed when `s` is.
pub struct Mapped<S, T, U>(
    /// \(O(1)\). The source sequence this view maps.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let a = [1, 2]; map(a, x => x * 2).src == a`
    pub src: S,
    f: T -> U,
)

impl Seq<U> for Mapped<S: Seq<T>, T, U>:
    fn iter(self) -> yields U:
        for x in src: yield f(x)

impl Keyed<D, U> for Mapped<S: Keyed<D, T>, T, U>:
    fn items(self) -> yields (D, U):
        for (k, x) in src.items: yield (k, f(x))

impl Sized for Mapped<S: Sized, T, U>:
    @get fn len(self) -> Int = src.len

impl Bidir<U> for Mapped<S: Bidir<T>, T, U>:
    fn iter_rev(self) -> yields U:
        for x in src.rev: yield f(x)

impl Positional<U> for Mapped<S: Indexed<D, T>, T, U>:
    fn nth(self, k: Int) -> U = f(src.nth(k))

impl IndexGet<D, U> for Mapped<S: Indexed<D, T>, T, U>:
    fn at(self, i: D) -> U = f(src.at(i))
impl Indexed<D, U> for Mapped<S: Indexed<D, T>, T, U>:
    @get fn keys(self) -> Range<D> = src.keys

/// \(O(1)\) to make. The view `zip(a, b)`: `a`'s elements paired with `b`'s by walk order, as many
/// as the shorter has, keyed by `a`'s keys (R04b.1.3).
pub struct Zipped<A, B, T, U>(
    /// \(O(1)\). The sequence whose walk and keys the pairs follow.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let a = [1, 2]; zip(a, "ab").first == a`
    pub first: A,
    /// \(O(1)\). The sequence read by position for each pair's second half.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `zip([1, 2], "ab").second == "ab"`
    pub second: B,
)

fn shorter(a: Int, b: Int) -> Int = if a < b: a else: b

impl Seq<(T, U)> for Zipped<A: Seq<T>, B: Positional<U>, T, U>:
    fn iter(self) -> yields (T, U):
        var i = 0
        for x in first:
            if i == second.len: return
            yield (x, second.nth(i))
            i += 1

impl Keyed<D, (T, U)> for Zipped<A: Keyed<D, T>, B: Positional<U>, T, U>:
    fn items(self) -> yields (D, (T, U)):
        var i = 0
        for (k, x) in first.items:
            if i == second.len: return
            yield (k, (x, second.nth(i)))
            i += 1

impl Sized for Zipped<A: Sized, B: Positional<U>, T, U>:
    @get fn len(self) -> Int = shorter(first.len, second.len)

impl Positional<(T, U)> for Zipped<A: Positional<T>, B: Positional<U>, T, U>:
    fn nth(self, k: Int) -> (T, U) = (first.nth(k), second.nth(k))

impl Bidir<(T, U)> for Zipped<A: Positional<T>, B: Positional<U>, T, U>:
    fn iter_rev(self) -> yields (T, U):
        for k in (0..shorter(first.len, second.len)).rev: yield (first.nth(k), second.nth(k))

impl IndexGet<D, (T, U)> for Zipped<A: Indexed<D, T>, B: Positional<U>, T, U>:
    fn at(self, i: D) -> (T, U) = (first.at(i), second.nth(i.ord - first.keys.start.ord))
impl Indexed<D, (T, U)> for Zipped<A: Indexed<D, T>, B: Positional<U>, T, U>:
    @get fn keys(self) -> Range<D> = first.keys.take(shorter(first.len, second.len))

/// \(O(1)\) to make. The view `s.rev`: `s` walked backwards, keys unchanged.
pub struct Rev<S>(base: S):
    /// \(O(1)\). The source sequence this view reverses.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let a = [1, 2]; a.rev.src == a`
    @get pub fn src(self) -> S = base

    /// \(O(1)\). The element `k` steps into the walk: `a.rev.nth(0)` is `a`'s last.
    ///
    /// cost: \(O(1)\)
    /// traps: unless `0 <= k < self.len`
    /// example: `[1, 2, 3].rev.nth(0) == 3`
    pub fn nth<T>(self, k: Int) -> T = self.src.nth(self.src.len - 1 - k)

    /// \(O(1)\). The cut after key `i` in this walk: the source's cut before it (R07.4.14).
    ///
    /// cost: \(O(1)\)
    /// traps: unless `i` is a key of the source (R0101)
    /// example: `domain V = 1..=4; let a = [V] v => Int(v); a.rev.cut_after(V(2)) == V(2).l`
    pub fn cut_after<D: Ordered, T>(self, i: D) -> Cut<D> where S: Indexed<D, T> = self.src.cut_before(i)

    /// \(O(1)\). The cut before key `i` in this walk: the source's cut after it (R07.4.14).
    ///
    /// cost: \(O(1)\)
    /// traps: unless `i` is a key of the source (R0101)
    /// example: `domain V = 1..=4; let a = [V] v => Int(v); a.rev.cut_before(V(2)) == V(2).r`
    pub fn cut_before<D: Ordered, T>(self, i: D) -> Cut<D> where S: Indexed<D, T> = self.src.cut_after(i)

    /// \(O(1)\). The first `min(k, len)` elements in walk order, as a reversed view:
    /// the source's last `k`, walked backwards.
    ///
    /// cost: \(O(1)\)
    /// empty: `let a: List<Int> = []; List(a.rev.take(2)) == []`
    /// order: walk order of `self`; keeps the source's keys
    /// traps: when `k < 0` or `k >= 2^31`
    /// example: `List([1, 2, 3].rev.take(2)) == [3, 2]`
    pub fn take(self, k: Len) -> Rev<Slice<S>> = self.src.take_last(k).rev

    /// \(O(1)\). All but the first `min(k, len)` elements in walk order, as a reversed view:
    /// the source without its last `k`, walked backwards.
    ///
    /// cost: \(O(1)\)
    /// empty: `let a: List<Int> = []; List(a.rev.drop(1)) == []`
    /// order: walk order of `self`; keeps the source's keys
    /// traps: when `k < 0` or `k >= 2^31`
    /// example: `List([1, 2, 3].rev.drop(1)) == [2, 1]`
    pub fn drop(self, k: Len = 1) -> Rev<Slice<S>> = self.src.drop_last(k).rev

    /// \(O(1)\). The last `min(k, len)` elements in walk order, as a reversed view:
    /// the source's first `k`, walked backwards.
    ///
    /// cost: \(O(1)\)
    /// empty: `let a: List<Int> = []; List(a.rev.take_last(2)) == []`
    /// order: walk order of `self`; keeps the source's keys
    /// traps: when `k < 0` or `k >= 2^31`
    /// example: `List([1, 2, 3].rev.take_last(2)) == [2, 1]`
    pub fn take_last(self, k: Len) -> Rev<Slice<S>> = self.src.take(k).rev

    /// \(O(1)\). All but the last `min(k, len)` elements in walk order, as a reversed view:
    /// the source without its first `k`, walked backwards.
    ///
    /// cost: \(O(1)\)
    /// empty: `let a: List<Int> = []; List(a.rev.drop_last(1)) == []`
    /// order: walk order of `self`; keeps the source's keys
    /// traps: when `k < 0` or `k >= 2^31`
    /// example: `List([1, 2, 3].rev.drop_last(1)) == [3, 2]`
    pub fn drop_last(self, k: Len = 1) -> Rev<Slice<S>> = self.src.drop(k).rev

impl Positional<T> for Rev<S: Indexed<D, T>>:
    fn nth(self, k: Int) -> T = self.src.nth(self.src.len - 1 - k)

impl Seq<T> for Rev<S: Bidir<T>>:
    fn iter(self) -> yields T:
        for x in self.src.iter_rev(): yield x

impl Bidir<T> for Rev<S: Bidir<T>>:
    fn iter_rev(self) -> yields T:
        for x in self.src: yield x

impl Sized for Rev<S: Sized>:
    @get fn len(self) -> Int = self.src.len

/// \(O(1)\) to make. The view `s.items`: `(key, element)` pairs in `s`'s order.
pub struct Items<S>(
    /// \(O(1)\). The source sequence whose items this view walks.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `let r = 0..3; r.items.src == r`
    pub src: S,
)

impl Seq<(D, T)> for Items<S: Keyed<D, T>>:
    fn iter(self) -> yields (D, T):
        for (k, x) in src.items(): yield (k, x)

impl Sized for Items<S: Sized>:
    @get fn len(self) -> Int = src.len

impl Keyed<D, (D, T)> for Items<S: Indexed<D, T>>:
    fn items(self) -> yields (D, (D, T)):
        for (k, x) in src.items: yield (k, (k, x))

impl Bidir<(D, T)> for Items<S: Indexed<D, T>>:
    fn iter_rev(self) -> yields (D, T):
        for k in src.keys.rev: yield (k, src.at(k))

impl Positional<(D, T)> for Items<S: Indexed<D, T>>:
    fn nth(self, k: Int) -> (D, T):
        let i = src.keys[k]
        (i, src.at(i))

impl IndexGet<D, (D, T)> for Items<S: Indexed<D, T>>:
    fn at(self, i: D) -> (D, T) = (i, src.at(i))
impl Indexed<D, (D, T)> for Items<S: Indexed<D, T>>:
    @get fn keys(self) -> Range<D> = src.keys

/// \(O(1)\) to make. `s[r]`: a lent view of `s` over the range `r`. Every slice keeps its
/// source's keys: `a[l..=r][i]` is `a[i]` and `i` must lie in `l..=r`; for a `List` or `Str`,
/// `s[2..5][3]` is `s[3]` and `s[2..5][0]` traps. `w.first`, `w.nth(k)` and `w.rebase()` give
/// offsets. Writable through a `var` source; the source's length can't change while it is alive.
/// See `spec/core/04b-keys.md`.
pub struct Slice<S>(
    /// The sequence the slice was cut from, lent: frozen while the slice is alive.
    src: S,
    /// The position in `src` of the slice's first element.
    lo: Int,
    /// The position in `src` just past the slice's last element; `lo <= hi <= src.len`.
    hi: Int,
):
    /// The number of elements in the slice.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3][1..].len == 2`
    @get pub fn len(self) -> Int = hi - lo
    /// Whether the slice has no elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2][2..].is_empty == true`
    @get pub fn is_empty(self) -> Bool = hi <= lo

/// \(O(1)\). The view without its first `k` elements, clamped (traps if `k < 0` or `k >= 2^31`). Consuming a view
/// from the front is `w = w.drop(1)`.
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.drop(1).collect() == []`
/// order: keeps the keys of `self`
/// traps: when `k < 0` or `k >= 2^31`
/// example: `[1, 2, 3].drop(1).collect() == [2, 3]`
@lint(interval_dp_by_length, from: "for len in 2..=n:\n    for l in 0..n - len + 1:\n        let r = l + len - 1\n        dp[l, r] = step", to: "for rg in I.ranges:\n    let (right, left) = (dp[rg.drop(1)], dp[rg.drop_last(1)])\n    dp[rg] = step", holes: "n, dp, step")
@lint(interval_dp_by_length, from: "for len in 2..=n:\n    for l in 0..n - len + 1:\n        let r = l + len - 1\n        dp[l][r] = step", to: "for rg in I.ranges:\n    let (right, left) = (dp[rg.drop(1)], dp[rg.drop_last(1)])\n    dp[rg] = step", holes: "n, dp, step")
pub fn drop<S: Indexed>(self: S, k: Len = 1) -> Slice<S> = piece(
    self,
    clamp_count(k, self.len),
    self.len,
)

/// \(O(1)\). The view without its last `k` elements, clamped (traps if `k < 0` or `k >= 2^31`).
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.drop_last(1).collect() == []`
/// order: keeps the keys of `self`
/// traps: when `k < 0` or `k >= 2^31`
/// example: `[1, 2, 3].drop_last(2).collect() == [1]`
pub fn drop_last<S: Indexed>(self: S, k: Len = 1) -> Slice<S> = piece(
    self,
    0,
    self.len - clamp_count(k, self.len),
)

/// \(O(1)\). The first `min(k, len)` elements, as a view (traps if `k < 0` or `k >= 2^31`).
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.take(2).collect() == []`
/// order: keeps the keys of `self`
/// traps: when `k < 0` or `k >= 2^31`
/// example: `[1, 2, 3].take(5).collect() == [1, 2, 3]`
pub fn take<S: Indexed>(self: S, k: Len) -> Slice<S> = piece(
    self,
    0,
    clamp_count(k, self.len),
)

/// \(O(1)\). The last `min(k, len)` elements, as a view (traps if `k < 0` or `k >= 2^31`).
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.take_last(2).collect() == []`
/// order: keeps the keys of `self`
/// traps: when `k < 0` or `k >= 2^31`
/// example: `[1, 2, 3].take_last(2).collect() == [2, 3]`
pub fn take_last<S: Indexed>(self: S, k: Len) -> Slice<S> = piece(
    self,
    self.len - clamp_count(k, self.len),
    self.len,
)

/// \(O(1)\). The element `k` steps from the start, 0-based whatever the keys: `s[l..].nth(k)` is
/// `s[l + k]`, `w.nth(0)` is `w.first!`. Traps when `k` is outside `0..len`.
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.nth(0) == 0` (traps R0101)
/// traps: when `k` is not in `0..self.len`
/// example: `[5, 6, 7][1..].nth(0) == 6`
pub fn nth<S: Indexed<D, T>, D: Domain, T>(self: S, k: Int) -> T = self.at(self.keys.nth(k))

impl<S: Indexed<K, T>, K: Domain, T> Seq<T> for Slice<S>:
    fn iter(self) -> yields T:
        for i in self.lo..self.hi: yield self.src.nth(i)

impl<S: Indexed<K, T>, K: Domain, T> Sized for Slice<S>:
    @get fn len(self) -> Int = self.hi - self.lo

impl<S: Indexed<K, T>, K: Domain, T> Keyed<K, T> for Slice<S>:
    /// \(O(\mathit{len})\). Visit every element with its source key; `for x in s` walks the
    /// same elements without the keys.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `[1, 2][2..].items.collect() == []`
    /// order: increasing key
    /// traps: never
    /// example: `[5, 6, 7][1..].items.collect() == [(1, 6), (2, 7)]`
    fn items(self) -> yields (K, T):
        for i in self.lo..self.hi: yield (self.src.keys.nth(i), self.src.nth(i))

impl<S: Indexed<K, T>, K: Domain, T> Bidir<T> for Slice<S>:
    fn iter_rev(self) -> yields T:
        for i in (self.lo..self.hi).rev: yield self.src.nth(i)

impl<S: Indexed<K, T>, K: Domain, T> Positional<T> for Slice<S>:
    /// `(0..len)[k]` is `k`, read as a key so one outside the slice traps R0101 (R07.4.5).
    fn nth(self, k: Int) -> T = self.src.nth(self.lo + (0..self.len)[k])

impl<S: Indexed<K, T>, K: Domain, T> IndexGet<K, T> for Slice<S>:
    /// `keys[i]` is `i`, checked: a key outside the slice traps R0101, never reading the source
    /// around it (R07.4.5).
    fn at(self, i: K) -> T = self.src.at(self.keys[i])
impl<S: Indexed<K, T>, K: Domain, T> Indexed<K, T> for Slice<S>:
    /// \(O(1)\). The slice's keys: a sub-range of the source's keys.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[5, 6, 7][1..].keys == 1..3`
    /// order: increasing
    @get fn keys(self) -> Range<K> = self.src.keys.drop(self.lo).take(self.hi - self.lo)

/// \(O(n)\). An owned copy of the slice's elements, in order.
///
/// cost: \(O(n)\)
/// traps: never
/// example: `[5, 6, 7][1..].to_list() == [6, 7]`
pub fn to_list<S: Indexed<K, T>, K, T>(self: Slice<S>) -> List<T> = self.collect()

/// \(O(1)\) to make. `g[r]`: row `r` of a grid `[R, C]T`, a lent view keyed by the column
/// domain, so its keys are `C`'s positions, not offsets from 0 (R04b.1.12).
pub struct Row<R, C, T>(
    /// The grid the row was cut from, lent: frozen while the row is alive.
    src: [R, C]T,
    /// Which row of `src` this is.
    r: R,
)

/// `g[r]`: row `r` of `g`, keyed by `C` (R04b.1.12).
@index fn row<R: Domain, C: Domain, T>(self: [R, C]T, r: R) -> Row<R, C, T> = Row(self, r)

impl<R: Domain, C: Domain, T> Seq<T> for Row<R, C, T>:
    fn iter(self) -> yields T:
        for c in C: yield self.src[self.r, c]

impl<R: Domain, C: Domain, T> Bidir<T> for Row<R, C, T>:
    fn iter_rev(self) -> yields T:
        for c in self.keys.rev: yield self.src[self.r, c]

impl<R: Domain, C: Domain, T> Sized for Row<R, C, T>:
    @get fn len(self) -> Int = C.count

impl<R: Domain, C: Domain, T> Keyed<C, T> for Row<R, C, T>:
    fn items(self) -> yields (C, T):
        for c in C: yield (c, self.src[self.r, c])

impl<R: Domain, C: Domain, T> Positional<T> for Row<R, C, T>:
    fn nth(self, k: Int) -> T = self.src[self.r, C.nth((0..C.count)[k])]

impl<R: Domain, C: Domain, T> IndexGet<C, T> for Row<R, C, T>:
    fn at(self, c: C) -> T = self.src[self.r, c]
impl<R: Domain, C: Domain, T> Indexed<C, T> for Row<R, C, T>:
    @get fn keys(self) -> Range<C> = C

/// `g[r, cs]`: the columns `cs` of row `r` of `g`, keyed by `C`; `g[r, ..]` is all of it
/// (R04.4.11).
@index fn row_piece<R: Domain, C: Domain, T>(
    self: [R, C]T,
    k: (R, Range<C>)
) -> Slice<Row<R, C, T>> = Row(self, k._0)[k._1]

/// \(O(1)\) to make. `g[.., c]`: column `c` of a grid `[R, C]T`, a lent view keyed by the row
/// domain, so its keys are `R`'s positions, not offsets from 0 (R04b.1.12).
pub struct Col<R, C, T>(
    /// The grid the column was cut from, lent: frozen while the column is alive.
    src: [R, C]T,
    /// Which column of `src` this is.
    c: C,
)

/// `g[rs, c]`: the rows `rs` of column `c` of `g`, keyed by `R`; `g[.., c]` is all of it
/// (R04.4.11).
@index fn col<R: Domain, C: Domain, T>(
    self: [R, C]T,
    k: (Range<R>, C)
) -> Slice<Col<R, C, T>> = Col(self, k._1)[k._0]

impl<R: Domain, C: Domain, T> Seq<T> for Col<R, C, T>:
    fn iter(self) -> yields T:
        for r in R: yield self.src[r, self.c]

impl<R: Domain, C: Domain, T> Bidir<T> for Col<R, C, T>:
    fn iter_rev(self) -> yields T:
        for r in self.keys.rev: yield self.src[r, self.c]

impl<R: Domain, C: Domain, T> Sized for Col<R, C, T>:
    @get fn len(self) -> Int = R.count

impl<R: Domain, C: Domain, T> Keyed<R, T> for Col<R, C, T>:
    fn items(self) -> yields (R, T):
        for r in R: yield (r, self.src[r, self.c])

impl<R: Domain, C: Domain, T> Positional<T> for Col<R, C, T>:
    fn nth(self, k: Int) -> T = self.src[R.nth((0..R.count)[k]), self.c]

impl<R: Domain, C: Domain, T> IndexGet<R, T> for Col<R, C, T>:
    fn at(self, r: R) -> T = self.src[r, self.c]
impl<R: Domain, C: Domain, T> Indexed<R, T> for Col<R, C, T>:
    @get fn keys(self) -> Range<R> = R

/// \(O(1)\) to make. `g[rs, cs]`: the rows `rs` and columns `cs` of a grid `[R, C]T`, a lent
/// view walked row-major, so `g[..n, ..m].sum()` folds a sub-rectangle (R04a.7.2).
pub struct Rect<R, C, T>(
    /// The grid the rectangle was cut from, lent: frozen while the rectangle is alive.
    src: [R, C]T,
    rs: Range<R>,
    cs: Range<C>,
)

/// `g[rs, cs]`: the rows `rs` and columns `cs` of `g`, its cells keyed by `(R, C)` (R04.4.11).
@index fn rect<R: Domain, C: Domain, T>(self: [R, C]T, k: (Range<R>, Range<C>)) -> Rect<R, C, T> =
    Rect(self, k._0, k._1)

impl<R: Domain, C: Domain, T> Seq<T> for Rect<R, C, T>:
    fn iter(self) -> yields T:
        for r in self.rs:
            for c in self.cs: yield self.src[r, c]

impl<R: Domain, C: Domain, T> Sized for Rect<R, C, T>:
    @get fn len(self) -> Int = self.rs.len * self.cs.len

impl<R: Domain, C: Domain, T> Keyed<(R, C), T> for Rect<R, C, T>:
    fn items(self) -> yields ((R, C), T):
        for r in self.rs:
            for c in self.cs: yield ((r, c), self.src[r, c])

/// \(O(1)\). `self` keyed by a fresh domain `0..len` of its own (`let t = w.rebase`, keys `t.Idx`),
/// for 0-based arithmetic on a piece; its keys can't be mixed with the source's.
///
/// cost: \(O(1)\)
/// empty: `let a: List<Int> = []; a.rebase.len == 0`
/// order: same elements, keyed `0..len` by a fresh domain
/// traps: never
/// example: `[5, 6, 7][1..].rebase.nth(1) == 7`
@get pub fn rebase<S: Indexed<K, T>, K, T>(self: S) -> Rebased<S> = Rebased(self)

/// The view `rebase` returns: `src` keyed by `Idx`, a fresh domain `0..src.len` of its own.
pub struct Rebased<S>(pub src: S):
    domain Idx = 0..src.len

impl<S: Indexed<K, T>, K: Domain, T> Seq<T> for Rebased<S>:
    fn iter(self) -> yields T:
        for x in self.src: yield x

impl<S: Sized> Sized for Rebased<S>:
    @get fn len(self) -> Int = self.src.len

impl<S: Indexed<K, T>, K: Domain, T> Keyed<Idx, T> for Rebased<S>:
    fn items(self) -> yields (Idx, T):
        for i in Idx.all: yield (i, self.src.nth(i.ord))

impl<S: Indexed<K, T>, K: Domain, T> Bidir<T> for Rebased<S>:
    fn iter_rev(self) -> yields T:
        for x in self.src.rev: yield x

impl<S: Indexed<K, T>, K: Domain, T> Positional<T> for Rebased<S>:
    fn nth(self, k: Int) -> T = self.src.nth(k)

impl<S: Indexed<K, T>, K: Domain, T> IndexGet<Idx, T> for Rebased<S>:
    fn at(self, i: Idx) -> T = self.src.nth(i.ord)
impl<S: Indexed<K, T>, K: Domain, T> Indexed<Idx, T> for Rebased<S>:
    @get fn keys(self) -> Range<Idx> = Idx.all

/// \(O(1)\). Every `k`-th element of `self`, starting with the first: `[3, 1, 2, 4].step(2)` walks
/// `3, 2`. Keys survive (R04a.9.3): `a.step(2)[i]` reads `a[i]`, and a key the stride skips traps.
///
/// cost: \(O(1)\) to make, \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.step(2).len == 0`
/// order: source order, every `k`-th; source keys
/// traps: when `k < 1`
/// example: `[3, 1, 2, 4, 5].step(2).nth(2) == 5`
pub fn step<S: Indexed<K, T>, K, T>(self: S, k: 1..) -> Stepped<S, T>:
    assert k >= 1, "step(k) requires k >= 1"
    Stepped(self, k)

/// \(O(1)\). Every other element, starting with the first: `s.step(2)`, with source keys.
///
/// cost: \(O(1)\) to make, \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.even.len == 0`
/// order: source order, every other element; source keys
/// traps: never
/// example: `[3, 1, 2, 4, 5].even.len == 3 and [3, 1, 2, 4, 5].even.nth(1) == 2`
@get pub fn even<S: Indexed<K, T>, K, T>(self: S) -> Stepped<S, T> = Stepped(self, 2)

/// \(O(1)\). Every other element, starting with the second: `s.even`'s complement, with source keys.
///
/// cost: \(O(1)\) to make, \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.odd.len == 0 and [7].odd.len == 0`
/// order: source order, every other element; source keys
/// traps: never
/// example: `[3, 1, 2, 4, 5].odd.len == 2 and [3, 1, 2, 4, 5].odd.nth(1) == 4`
@get pub fn odd<S: Indexed<K, T>, K, T>(self: S) -> Stepped<S, T> = Stepped(self, 2, from: 1)

/// \(O(1)\) to make. The view `s.step(k)`: `s` walked `k` elements at a time from position
/// `from`, keys unchanged.
pub struct Stepped<S, T>(pub src: S, pub k: Int, pub from: Int = 0):
    /// \(O(1)\). How many elements the walk visits: `ceil((src.len - from) / k)`, or 0.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[3, 1, 2, 4, 5].step(2).len == 3`
    @get pub fn len(self) -> Int = if from >= src.len: 0 else: (src.len - from + k - 1) / k

    /// Whether the stepped view has no elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2][2..].step(2).is_empty and not [1].step(3).is_empty`
    @get pub fn is_empty(self) -> Bool = self.len == 0

    /// \(O(\mathit{len})\). Visit the elements `src.nth(from)`, `src.nth(from + k)`, ...
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `[1, 2][2..].step(2).each().collect() == []`
    /// order: increasing position in `src`
    /// traps: never
    /// example: `[3, 1, 2, 4, 5].step(2).each().collect() == [3, 2, 5]`
    pub fn each(self) -> yields T:
        for j in 0..self.len: yield self.nth(j)

    /// \(O(1)\). The element `j` steps into the walk: `src`'s element `from + j * k`.
    ///
    /// cost: \(O(1)\)
    /// traps: unless `0 <= j < self.len`
    /// example: `[3, 1, 2, 4].step(2).nth(1) == 2`
    pub fn nth(self, j: Int) -> T = src.nth(from + j * k)

    /// \(O(1)\). The element at source key `i` (R04a.9.3): `a.step(2).at(i)` is `a[i]`, and
    /// `a.step(2)[i]` reads it.
    ///
    /// cost: \(O(1)\)
    /// traps: unless `i` is a key the stride visits (`first`, `first + k`, ...), naming them
    /// example: `[10, 20, 30, 40, 50].step(2).at(4) == 50`
    pub fn at<K>(self, i: K) -> T:
        assert i in self.keys
        src.at(i)

    /// \(O(1)\). The source keys the walk visits: `i in a.step(2).keys` tests the stride.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `4 in [10, 20, 30, 40, 50].step(2).keys and not (1 in [10, 20, 30, 40, 50].step(2).keys)`
    @get pub fn keys(self) -> StepKeys:
        StepKeys(if src.len == 0: 0 else: src.keys.first!.ord, src.len, k, from)

/// The keys a `step(k)` view visits, made by its `keys`: of the `n` source keys from position
/// `first`, every `k`-th one from `from` on.
///
/// not a container: a membership test for `in`; walk the step view itself for its keys and elements.
pub struct StepKeys(first: Int, n: Int, k: Int, from: Int):
    /// \(O(1)\). How many keys the walk visits.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[3, 1, 2, 4, 5].step(2).keys.len == 3`
    @get pub fn len(self) -> Int = if from >= n: 0 else: (n - from + k - 1) / k

impl<K: Domain> Contains<K> for StepKeys:
    /// \(O(1)\). Whether the walk visits `i`: `i in s.keys`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[3, 1, 2, 4, 5].step(2).keys.contains(4)`
    fn contains(self, i: K) -> Bool:
        let o = i.ord - first - from
        0 <= o < n - from and o % k == 0

/// \(O(n)\). The same cells keyed by `keys`, which must have the same length (traps otherwise):
/// a table computed on offsets handed back in the caller's keys, `z.rekey(s.keys)`.
///
/// cost: \(O(n)\), allocates
/// empty: `let a: List<Int> = []; a.rekey(5..5).len == 0`
/// order: same cells, keyed by `keys`
/// traps: when `keys.len != self.len`
/// example: `[4, 5].rekey(10..12)[11] == 5`
@lint(query_index_shift, from: "let i: 1..=n = read()", advice: "domain I = 1..=n", holes: "n", also: "self[i - 1]")
@lint(query_index_shift, from: "let (i, j): (1..=n, T) = read()", advice: "domain I = 1..=n", holes: "n, T", also: "self[i - 1]")
@lint(query_index_shift, from: "let (j, i): (T, 1..=n) = read()", advice: "domain I = 1..=n", holes: "n, T", also: "self[i - 1]")
@lint(query_index_shift, from: "let (i, j, k): (1..=n, T, U) = read()", advice: "domain I = 1..=n", holes: "n, T, U", also: "self[i - 1]")
@lint(query_index_shift, from: "let (j, i, k): (T, 1..=n, U) = read()", advice: "domain I = 1..=n", holes: "n, T, U", also: "self[i - 1]")
@lint(query_index_shift, from: "let (j, k, i): (T, U, 1..=n) = read()", advice: "domain I = 1..=n", holes: "n, T, U", also: "self[i - 1]")
pub fn rekey<D, K, T>(self: [D]T, keys: Range<K>) -> [K]T:
    assert self.len == keys.len, "`rekey` needs one key per element"
    [K] k => self.nth(k.ord - keys.start.ord)

/// The end of a lazy chain, materialized into the type the context expects, else a `List`.
/// \(O(n)\), allocates.
///
/// The context may want a `List`, `Set`, `Map`, `Str` or `[D]T`: `a.filter(p).map(f).collect()`.
/// The same operation as `List(s)`; the postfix form is for chains.
///
/// cost: \(O(n)\), allocates
/// empty: `let a: List<Int> = []; a.collect() == []`
/// order: input order; the container the context expects, else a `List`
/// traps: never
/// example: `(1..4).map(x => x * x).collect() == [1, 4, 9]`
pub fn collect<S: Seq<T>, T>(self: S) -> List<T>:
    let out: List<T> = [x for x in self]
    out

/// `self` with keys attached, so it is `Keyed`: walking it still gives values, `.items`
/// gives `(key, x)`, and `argmax`/`find` answer with keys.
///
/// By default the k-th element gets
/// the k-th key of `keys` (`0..` is a counter: `for (i, x) in g.keyed().items`); with `by:`
/// each key is `by(x)` (`dfs(s).keyed(by: v => v)`). Passing both is an error. \(O(1)\) to make.
///
/// cost: \(O(1)\) to make, \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.keyed().items.collect() == []`
/// order: input order, with the k-th key of `keys` or `by(x)`
/// traps: when both `keys` and `by` are given; when `keys` runs out first
/// example: `"ab".keyed().items.collect() == [(0, 'a'), (1, 'b')]`
pub fn keyed<S: Seq<T>, I: Indexed, T, K>(
    self: S,
    keys: I = 0..,
    by: ?(T -> K) = None,
) -> Keys<S, I, T, K> = Keys(self, keys, by)

/// The view `keyed` returns.
pub struct Keys<S, I, T, K>(pub src: S, pub keys: I, by: ?(T -> K))

impl Seq<T> for Keys<S, I, T, K>:
    fn iter(self) -> yields T:
        for x in src: yield x

impl Keyed<K, T> for Keys<S, I, T, K>:
    fn items(self) -> yields (K, T):
        var j = 0
        for x in src:
            if by != None: yield (by!(x), x)
            else: yield (keys.nth(j), x)
            j += 1

/// \(O(1)\) per step. Yield leading elements while `keep(x)` holds, stopping at the first failure.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.take_while(x => x < 3).count() == 0`
/// order: input order
/// traps: never
/// example: `[1, 2, 5, 1].take_while(x => x < 3).collect() == [1, 2]`
pub fn take_while<S: Seq<T>, T>(self: S, keep: T -> Bool) -> yields T:
    for x in self:
        if not keep(x): return
        yield x

/// \(O(1)\) per step. Skip leading elements while `drop(x)` holds, then yield the rest.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.drop_while(x => x < 3).count() == 0`
/// order: input order
/// traps: never
/// example: `[1, 2, 5, 1].drop_while(x => x < 3).collect() == [5, 1]`
pub fn drop_while<S: Seq<T>, T>(self: S, drop: T -> Bool) -> yields T:
    var dropping = true
    for x in self:
        if dropping and drop(x): continue
        dropping = false
        yield x

/// \(O(1)\) per step. Lazily yield `f(x)` for each element where it is not `None`:
/// `dag[c].filter_map(a => best(a.to)).max()`.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.filter_map(x => Some(x)).count() == 0`
/// order: input order
/// traps: never
/// example: `[1, 2, 3].filter_map(x => if x != 2: Some(x * 10) else: None).collect() == [10, 30]`
pub fn filter_map<S: Seq<T>, T, U>(self: S, f: T -> ?U) -> yields U:
    for x in self:
        if let y = f(x): yield y

/// \(O(1)\) per yielded element. Lazily map each element to a sequence and yield all inner elements in order.
///
/// cost: \(O(1)\) per yielded element
/// empty: `let a: List<Int> = []; a.flat_map(x => 0..x).count() == 0`
/// order: outer order, then each inner sequence's order
/// traps: never
/// example: `[1, 2].flat_map(x => 0..x).collect() == [0, 0, 1]`
pub fn flat_map<S: Seq<T>, T, U: Seq<V>, V>(self: S, f: T -> U) -> yields V:
    for x in self:
        for y in f(x): yield y

/// \(O(1)\) per yielded element. Lazily concatenate a sequence of sequences.
///
/// cost: \(O(1)\) per yielded element
/// empty: `let a: List<List<Int>> = []; a.flatten().count() == 0`
/// order: outer order, then inner order
/// traps: never
/// example: `[[1], [], [2, 3]].flatten().collect() == [1, 2, 3]`
pub fn flatten<S: Seq<U>, U: Seq<T>, T>(self: S) -> yields T:
    for xs in self:
        for x in xs: yield x

/// \(O(1)\) per step. Yield the `Some` values of a sequence of `?T`, skipping `None`.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<?Int> = []; a.present().count() == 0`
/// order: input order
/// traps: never
/// example: `[Some(1), None, Some(3)].present().collect() == [1, 3]`
pub fn present<S: Seq<?U>, U>(self: S) -> yields U:
    for x in self:
        if let y = x: yield y

/// \(O(n)\), \(O(n)\) memory. Unwrap every `?T` into a `List<T>`, or return `None` if any element is `None`.
///
/// cost: \(O(n)\), \(O(n)\) memory
/// empty: `let a: List<?Int> = []; a.all_some() == Some([])`
/// order: input order
/// traps: never
/// example: `[Some(1), None].all_some() == None`
pub fn all_some<S: Seq<?U>, U>(self: S) -> ?List<U>:
    var out: List<U> = []
    for x in self:
        if let y = x: out.push(y)
        else: return None
    Some(out)

/// \(O(1)\) per step. Lazily concatenate any number of sequences, in turn, without copying
/// (R07.7.4), as free `min` and `max` take any number of values.
///
/// cost: \(O(1)\) per step
/// empty: `let e: List<Int> = []; chain(e, [1], e, [2]).collect() == [1, 2]`
/// order: all of the first sequence, then all of the next, and so on
/// traps: never
/// example: `chain([1], [2, 3]).collect() == [1, 2, 3]`
/// example: `chain([1], [2], [3, 4]).collect() == [1, 2, 3, 4]`
pub fn chain<Ss, T>(..seqs: Ss) -> yields T:
    for s in fields(seqs):
        for x in s: yield x

/// \(O(1)\) per step. Cartesian product `(x, y)` for `x in a`, `y in b` in lexicographic order.
///
/// cost: \(O(1)\) per step, \(O(\lvert a \rvert \cdot \lvert b \rvert)\) to walk
/// empty: `let e: List<Int> = []; product(e, "xy").count() == 0 and product([1, 2], "").count() == 0`
/// order: lexicographic, `a` outer
/// traps: never
/// example: `product([1, 2], "xy").count() == 4`
pub fn product<A: Seq<T>, B: Indexed<L, U>, L, T, U>(a: A, b: B) -> yields (T, U):
    for x in a:
        for y in b: yield (x, y)

/// \(O(1)\) per step. One-shot stream `from, step(from), step(step(from)), ...` until `None`.
///
/// cost: \(O(1)\) per step plus one call of `step`
/// empty: `let s: ?Int = None; iterate(s, x => Some(x + 1)).count() == 0`
/// order: `from`, `step(from)`, ... until `None`
/// traps: never
/// example: `iterate(Some(1), x => if x < 8: Some(2 * x) else: None).collect() == [1, 2, 4, 8]`
pub fn iterate<T>(from: ?T, step: T -> ?T) -> yields T:
    var cur = from
    while let x = cur:
        yield x
        cur = step(x)

/// \(O(1)\) per step. Every element paired with the next one, ending with `(last, None)`.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; a.with_next().count() == 0`
/// order: left to right, ending `(last, None)`
/// traps: never
/// example: `[1, 2].with_next().collect() == [(1, Some(2)), (2, None)]`
pub fn with_next<S: Seq<T>, T>(self: S) -> yields (T, ?T):
    var prev: ?T = None
    for x in self:
        if let p = prev: yield (p, Some(x))
        prev = Some(x)
    if let p = prev: yield (p, None)

/// \(O(1)\) per step. Zero-alloc zipped view of equal-length sequences; traps if `a.len != b.len`.
///
/// cost: \(O(1)\) per step
/// empty: `let a: List<Int> = []; zip_exact(a, "").count() == 0`
/// order: position by position
/// traps: when `a.len != b.len`
/// example: `zip_exact([1, 2], "ab").collect() == [(1, 'a'), (2, 'b')]`
pub fn zip_exact<A: Indexed<K, T>, B: Positional<U>, K, T, U>(a: A, b: B) -> yields (T, U):
    assert a.len == b.len
    for i in 0..a.len: yield (a.nth(i), b.nth(i))

/// \(O(n)\). Non-overlapping length-`k` chunks, padding the last chunk with `pad` when shorter.
///
/// cost: \(O(k)\) per chunk
/// empty: `let a: List<Int> = []; a.chunks_padded(2, 0).count() == 0`
/// order: left to right
/// traps: when `k < 1`
/// example: `[1, 2, 3].chunks_padded(2, 0).collect() == [[1, 2], [3, 0]]`
pub fn chunks_padded<S: Indexed<D, T>, D: Domain, T>(self: S, k: 1.., pad: T) -> yields List<T>:
    assert k >= 1
    var i = 0
    while i < self.len:
        let c: List<T> = [if j < self.len: self.nth(j) else: pad for j in i..i + k]
        yield c
        i += k

// --- Reductions, folds, frequencies, and selection ---

/// \(O(n)\). Whether some element equals `x`. A type with a faster `contains` of its own (a set,
/// a map, a bag, a hashed or keyed type) declares it, and its declared member wins; only those
/// sub-linear `Contains` types take `x in s` (R12.4.7).
///
/// cost: \(O(n)\) comparisons
/// empty: `let a: List<Int> = []; a.contains(3) == false`
/// traps: never
/// example: `(1..=3).map(x => x * x).contains(4)`
pub fn contains<S: Seq<T>, T: Eq>(self: S, x: T) -> Bool:
    for y in self:
        if y == x: return true
    false


/// \(O(n)\). Seedless left fold returning `None` on empty (never traps).
///
/// cost: \(O(n)\) calls of `f`
/// empty: `let a: List<Int> = []; a.reduce((x, y) => x * y) == None`
/// traps: never
/// example: `[3, 4].reduce((a, b) => a * b) == Some(12)`
pub fn reduce<S: Seq<T>, T>(self: S, f: (T, T) -> T) -> ?T:
    var acc: ?T = None
    for x in self:
        if let a = acc: acc = Some(f(a, x))
        else: acc = Some(x)
    acc

/// \(O(n)\). The elements folded left to right with `op`: `op(op(op(id, x0), x1), x2)`. `op` is
/// anything `Monoid.of` accepts (`+`, `*`, `^`, `|`, `&`, `max`, `min`, `gcd`, `or`, `and`, a
/// `Monoid`), so the empty fold is its identity. A fn without an identity is `reduce(f)`, and a fold
/// into another type is `fold_from(init, f)` (decision 550).
///
/// cost: \(O(n)\) calls of `op`
/// empty: `let a: List<Int> = []; a.fold(+) == 0`
/// traps: never (an `op` without a known identity is a compile-time error)
/// example: `[1, 2, 3].fold(+) == 6`
/// example: `[3, 9, 6].fold(gcd) == 3`
pub fn fold<S: Seq<T>, T, M: FromOp<T>>(self: S, op: M) -> T:
    let m = Monoid<T>.of(op)
    var acc = m.id
    for x in self: acc = m.op(acc, x)
    acc

/// \(O(n)\). Seeded left fold into an arbitrary accumulator `Acc`, `init` first.
///
/// cost: \(O(n)\) calls of `f`
/// empty: `let a: List<Int> = []; a.fold_from(7, (s, x) => s * 10 + x) == 7`
/// traps: never
/// example: `[1, 2, 3].fold_from(0, (s, x) => s * 10 + x) == 123`
pub fn fold_from<S: Seq<T>, T, Acc>(self: S, init: Acc, f: (Acc, T) -> Acc) -> Acc:
    var acc = init
    for x in self: acc = f(acc, x)
    acc

/// \(O(n)\). Maximum element (under `by:`) together with its multiplicity, or `None` when empty.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.max_count() == None`
/// ties: `[(3, 'a'), (1, 'b'), (3, 'c')].max_count(by: .0) == Some((val: (3, 'a'), count: 2))`
/// traps: never
/// example: `[3, 1, 3].max_count() == Some((val: 3, count: 2))`
pub fn max_count<S: Seq<T>, T, K: Ord>(self: S, by: T -> K = x => x) -> ?(val: T, count: Int):
    var best: ?(T, K) = None
    var count = 0
    for x in self:
        let k = by(x)
        if let (_, b) = best:
            if b < k:
                best = Some((x, k))
                count = 1
            elif k == b: count += 1
        else:
            best = Some((x, k))
            count = 1
    if let (v, _) = best: Some((val: v, count:)) else: None

/// \(O(n)\). Minimum element (under `by:`) together with its multiplicity, or `None` when empty.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.min_count() == None`
/// ties: `[(1, 'a'), (3, 'b'), (1, 'c')].min_count(by: .0) == Some((val: (1, 'a'), count: 2))`
/// traps: never
/// example: `[1, 2, 1].min_count() == Some((val: 1, count: 2))`
pub fn min_count<S: Seq<T>, T, K: Ord>(self: S, by: T -> K = x => x) -> ?(val: T, count: Int):
    var best: ?(T, K) = None
    var count = 0
    for x in self:
        let k = by(x)
        if let (_, b) = best:
            if k < b:
                best = Some((x, k))
                count = 1
            elif k == b: count += 1
        else:
            best = Some((x, k))
            count = 1
    if let (v, _) = best: Some((val: v, count:)) else: None

/// The value held by more than half the elements, with its count, or `None`.
///
/// Only `eq` is asked, never order or hashing, so it works with an equality oracle (Fischer-Salzberg: one
/// pass builds a chain with no two neighbours equal, a second counts its last value).
///
/// cost: at most \(\lfloor 3n/2 \rfloor\) calls of `eq`; \(O(n)\) memory
/// empty: `let a: List<Int> = []; a.majority() == None`
/// traps: never
/// example: `[2, 1, 2].majority() == Some((val: 2, count: 2))`
pub fn majority<S: Seq<T>, T>(self: S, eq: (T, T) -> Bool = (a, b) => a == b) -> ?(val: T, count: Int):
    var chain: List<T> = []
    var same: List<T> = []
    for x in self:
        if chain.len == 0 or not eq(chain.last!, x):
            chain.push(x)
            if let y = same.pop(): chain.push(y)
        else: same.push(x)
    let c = chain.last else: return None
    var spare = same.len
    var count = same.len + 1
    var i = chain.len - 3
    while i >= 0:
        if eq(c, chain[i]):
            count += 1
            i -= 2
        elif spare == 0: return None
        else:
            spare -= 1
            i -= 1
    if 2 * count > chain.len + same.len: Some((val: c, count:)) else: None

/// \(O(n \log n)\), \(O(n)\) memory. Group elements by an ordered key; `of:` keeps a projection of each element (`pairs.group(by: .0, of: .1)`). When the key is a domain, `a.map(key).buckets()` keys the result by that domain instead.
///
/// cost: \(O(n \log n)\), \(O(n)\) memory
/// empty: `let a: List<(Int, Char)> = []; a.group(by: p => p.0).len == 0`
/// order: keys ascending; each group keeps input order
/// traps: never
/// example: `[(1, 'a'), (2, 'b'), (1, 'c')].group(by: p => p.0, of: p => p.1)[1] == ['a', 'c']`
@lint(group_into_map, from: "for x in self:\n    (m[k] ??= []).push(v)", to: "let m = self.group(by: x => k, of: x => v)", holes: "m, k, v")
pub fn group<S: Seq<T>, T, K: Ord, U>(self: S, by: T -> K, of: T -> U = x => x) -> Map<K, List<U>>:
    var out = Map<K, List<U>>()
    for x in self:
        let k = by(x)
        if out.contains(k): out[k].push(of(x))
        else: out[k] = [of(x)]
    out

/// \(O(n)\) expected, \(O(k)\) memory. `k` smallest elements under `by:` (unordered; ties to earlier index).
///
/// cost: \(O(n)\) expected, \(O(k)\) memory: one pass into a buffer of at most `2k`, cut back to `k` by `select` whenever it fills
/// empty: `let a: List<Int> = []; a.min_k(2) == []`
/// ties: `[(1, 'a'), (1, 'b'), (1, 'c')].min_k(2, by: .0).sorted() == [(1, 'a'), (1, 'b')]`
/// order: unspecified; sort it when order matters
/// traps: when `k < 0` or `k >= 2^31`; `k > n` gives every element
/// example: `[5, 1, 4, 2].min_k(2).sorted() == [1, 2]`
pub fn min_k<S: Seq<T>, T, K: Ord>(self: S, k: Len, by: T -> K = x => x) -> List<T>:
    keep_k(self, k, by, (p, q) => p.0 < q.0 or (p.0 == q.0 and p.1 < q.1))

/// \(O(n)\) expected, \(O(k)\) memory. `k` largest elements under `by:` (unordered; ties to earlier index).
///
/// cost: \(O(n)\) expected, \(O(k)\) memory: one pass into a buffer of at most `2k`, cut back to `k` by `select` whenever it fills
/// empty: `let a: List<Int> = []; a.max_k(2) == []`
/// ties: `[(1, 'a'), (1, 'b'), (1, 'c')].max_k(2, by: .0).sorted() == [(1, 'a'), (1, 'b')]`
/// order: unspecified; sort it when order matters
/// traps: when `k < 0` or `k >= 2^31`; `k > n` gives every element
/// example: `[5, 1, 4, 2].max_k(2).sorted() == [4, 5]`
pub fn max_k<S: Seq<T>, T, K: Ord>(self: S, k: Len, by: T -> K = x => x) -> List<T>:
    keep_k(self, k, by, (p, q) => q.0 < p.0 or (p.0 == q.0 and p.1 < q.1))

// the `k` first under `first` of `(key, index, element)`, in a buffer that never passes `2k`
fn keep_k<S: Seq<T>, T, K>(self: S, k: Int, by: T -> K, first: ((K, Int, T), (K, Int, T)) -> Bool) -> List<T>:
    if k == 0: return []
    var ps: List<(K, Int, T)> = []
    var i = 0
    for x in self:
        ps.push((by(x), i, x))
        i += 1
        if ps.len == 2 * k:
            select(mut ps, k - 1, cmp: first)
            ps.truncate(k)
    if k < ps.len:
        select(mut ps, k - 1, cmp: first)
        ps.truncate(k)
    [p.2 for p in ps]

/// \(O(n)\). Unstable in-place partition moving `keep(x)` elements to `0..mid`; returns `mid`.
///
/// cost: \(O(n)\), \(O(1)\) extra space
/// empty: `var a: List<Int> = []; a.partition_in_place(x => x > 0) == 0`
/// order: kept elements in `0..mid`, the rest after, each side in unspecified order
/// traps: never
/// example: `var a = [1, 2, 3, 4]; a.partition_in_place(x => x % 2 == 0) == 2`
pub fn partition_in_place<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, keep: T -> Bool) -> Int:
    var mid = 0
    for i in 0..self.len:
        if keep(self.nth(i)):
            self.swap_idx(mid, i)
            mid += 1
    mid

/// \(O(n)\), \(O(n)\) memory. Stable in-place partition preserving relative order in both halves; returns `mid`.
///
/// cost: \(O(n)\), \(O(n)\) memory
/// empty: `var a: List<Int> = []; a.stable_partition_in_place(x => x > 0) == 0`
/// order: kept elements in `0..mid`, the rest after, each side in input order
/// traps: never
/// example: `var a = [1, 2, 3, 4]; a.stable_partition_in_place(x => x % 2 == 0); a == [2, 4, 1, 3]`
pub fn stable_partition_in_place<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, keep: T -> Bool) -> Int:
    var yes: List<T> = []
    var no: List<T> = []
    for i in 0..self.len:
        let x = self.nth(i)
        if keep(x): yes.push(x) else: no.push(x)
    let mid = yes.len
    for i in 0..mid: self[i] = yes[i]
    for i in 0..no.len: self[mid + i] = no[i]
    mid

// --- Differences and cut-indexed forms (decision 124) ---

/// \(O(n)\), \(O(n)\) memory. `[a[0], a[1] - a[0], ...]` (inverse of `scan(+)`).
///
/// cost: \(O(n)\), \(O(n)\) memory
/// empty: `let a: List<Int> = []; a.adjacent_diff() == []`
/// order: a new `List`, keyed from 0
/// traps: never
/// example: `[3, 5, 4].adjacent_diff() == [3, 2, -1]`
pub fn adjacent_diff<S: Seq<T>, T: Sub>(self: S) -> List<T>:
    var out: List<T> = []
    var prev: ?T = None
    for x in self:
        if let p = prev: out.push(x - p)
        else: out.push(x)
        prev = Some(x)
    out

/// \(O(1)\). Explicit piece `Range<D>` between cuts `lo` and `hi` (explicit twin of `lo..hi` in decision 124; traps if `lo > hi`).
///
/// cost: \(O(1)\)
/// traps: when `lo > hi`
/// example: `domain D = 0..4; span(D.cuts.first!, D.cuts.last!) == D.all`
/// order: increasing, from `lo` to `hi`
pub fn span<D>(lo: Cut<D>, hi: Cut<D>) -> Range<D>:
    assert Int(lo) <= Int(hi)
    lo..hi

// --- Sorted lookup, element bsearch, sorted set ops, and linear-time sorts ---

/// \(O(\log n)\). The cut where monotone `ok` flips on the elements of `self`: false then true by
/// default, true then false with `last: true`, as `Range.bsearch` is for positions (total; never `None`).
///
/// An `ok` that flips the other way (C++'s `partition_point` habit) stops a checked build.
/// Ties can't arise: a monotone `ok` flips once.
///
/// cost: \(O(\log n)\) calls of `ok`, plus one when the cut is an end
/// empty: `let a: List<Int> = []; a.bsearch(x => x > 2).r == 0`
/// traps: when `ok` flips the other way; any other `ok` that is not monotone gives an
/// unspecified answer
/// example: `[1, 3, 5].bsearch(x => x > 2).r == 1`
/// ties: `[1, 3, 3, 5].bsearch(x => x >= 3).r == 1`
@aka(partition_point)
pub fn bsearch<D, T>(self: [D]T, ok: T -> Bool, last: Bool = false) -> Cut<D>:
    let at = m => ok(self[D.nth(m)])
    D.cuts.nth(if last: ((0..D.count).last_true(at) ?? -1) + 1 else: (0..D.count).first_true(at) ?? D.count)

/// \(O(\log n)\). Contiguous range of elements in sorted `self` with `by(x) == target` (empty at insertion cut if absent).
///
/// cost: \(O(\log n)\)
/// empty: `let a: List<Int> = []; a.equal_range(3) == 0..0`
/// order: the keys of `self` holding `target`, increasing
/// traps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;
/// otherwise unsorted input gives an unspecified range
/// example: `[1, 3, 3, 5].equal_range(3) == 1..3`
pub fn equal_range<D, T, K: Ord>(self: [D]T, target: K, by: T -> K = x => x) -> Range<D>:
    let lo = (0..D.count).first_true(m => by(self[D.nth(m)]) >= target) ?? D.count
    let a = (lo..D.count).first_true(m => by(self[D.nth(m)]) > target) ?? D.count
    D.cuts.nth(lo)..D.cuts.nth(a)

/// \(O(\log n)\). How many elements of sorted `self` have `by(x)` in the half-open `r`.
///
/// cost: \(O(\log n)\)
/// empty: `let a: List<Int> = []; a.count_in(3..6) == 0 and [1, 3].count_in(3..3) == 0`
/// traps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;
/// otherwise unsorted input gives an unspecified count
/// example: `[1, 3, 3, 5, 8].count_in(3..6) == 3`
/// example: `[1, 3, 3, 5, 8].count_in(4..5) == 0`
pub fn count_in<D, T, K>(self: [D]T, r: Range<K>, by: T -> K = x => x) -> Int:
    if r.r <= r.l: return 0
    let lo = (0..D.count).first_true(m => by(self[D.nth(m)]) >= r.l) ?? D.count
    let hi = (lo..D.count).first_true(m => by(self[D.nth(m)]) >= r.r) ?? D.count
    hi - lo

/// \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\), \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\) memory. Sorted deduplicated union of two sorted sequences.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\), \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\) memory
/// empty: `let e: List<Int> = []; e.sorted_union([2, 2, 3]) == [2, 3]`
/// ties: `[1, 1, 3].sorted_union([1, 3, 3]) == [1, 3]`
/// order: increasing; a new `List`, keyed from 0
/// traps: never
/// example: `[1, 3].sorted_union([2, 3]) == [1, 2, 3]`
pub fn sorted_union<A: Indexed, B: Indexed, T: Eq>(self: A, b: B) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < self.len or j < b.len:
        let x = if j == b.len or (i < self.len and self.nth(i) < b.nth(j)): self.nth(i) else: b.nth(j)
        if out.len == 0 or out.last! != x: out.push(x)
        if i < self.len and self.nth(i) == x: i += 1
        else: j += 1
    out

/// The values in both of two sorted sequences, merged in one pass.
/// Each value is kept min(count in `self`, count in `b`) times.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\), \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\) memory
/// empty: `let e: List<Int> = []; e.sorted_intersect([2, 2]) == []`
/// ties: `[2, 2, 2].sorted_intersect([2, 2]) == [2, 2]`
/// order: increasing; a new `List`, keyed from 0
/// traps: never
/// example: `[1, 2, 2, 3].sorted_intersect([2, 2, 4]) == [2, 2]`
pub fn sorted_intersect<A: Indexed, B: Indexed, T>(self: A, b: B) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < self.len and j < b.len:
        if self.nth(i) < b.nth(j): i += 1
        elif b.nth(j) < self.nth(i): j += 1
        else:
            out.push(self.nth(i))
            i += 1
            j += 1
    out

/// \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\), \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\) memory. Sorted elements in `self` not in `b`.
/// Each value is kept (count in `self` − count in `b`) times, at least 0.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\), \(O(\lvert \mathit{self} \rvert + \lvert b \rvert)\) memory
/// empty: `let e: List<Int> = []; [1, 1, 2].sorted_diff(e) == [1, 1, 2]`
/// ties: `[2, 2, 3].sorted_diff([2, 2, 2]) == [3]`
/// order: increasing; a new `List`, keyed from 0
/// traps: never
/// example: `[1, 2, 2, 3].sorted_diff([2]) == [1, 2, 3]`
pub fn sorted_diff<A: Indexed, B: Indexed, T>(self: A, b: B) -> List<T>:
    var out: List<T> = []
    var j = 0
    for i in 0..self.len:
        let x = self.nth(i)
        while j < b.len and b.nth(j) < x: j += 1
        if j < b.len and b.nth(j) == x: j += 1
        else: out.push(x)
    out

/// \(O(N \log k)\), \(O(N)\) memory. Merge `k` sorted sequences via min-heap.
/// On ties the earlier list comes first (stable).
///
/// cost: \(O(N \log k)\), \(O(N)\) memory
/// empty: `let e: List<List<Int>> = []; kway_merge(e) == []`
/// ties: `kway_merge([[1, 2], [1]]) == [1, 1, 2]`
/// order: non-decreasing; a new `List`, keyed from 0
/// traps: never
/// example: `kway_merge([[1, 4], [2], [3]]) == [1, 2, 3, 4]`
pub fn kway_merge<S: Indexed<D, T>, D: Domain, T>(lists: List<S>) -> List<T>:
    if lists.len == 0: return []
    var parts: List<List<T>> = [l.collect() for l in lists]
    while parts.len > 1:
        var next: List<List<T>> = []
        var i = 0
        while i + 1 < parts.len:
            next.push(merge_sorted(parts[i], parts[i + 1]).collect())
            i += 2
        if i < parts.len: next.push(parts[i].clone())
        parts = next
    parts[0].clone()

/// \(O(n)\), \(O(n)\) memory. Stable LSD radix sort in place by integer key `by:` (signed `Int` supported).
///
/// cost: \(O(n)\) passes over 8-bit digits of the key, \(O(n)\) memory
/// empty: `var a: List<Int> = []; a.radix_sort(); a == []`
/// ties: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.radix_sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`
/// traps: never
/// example: `var a = [3, -1, 2]; a.radix_sort(); a == [-1, 2, 3]`
pub fn radix_sort<S: IndexedMut<D, T>, D: Domain, T>(mut self: S, by: T -> Int = x => x):
    let n = self.len
    if n <= 1: return
    var cur: List<(Int, T)> = [(by(self.nth(i)), self.nth(i)) for i in 0..n]
    var lo = cur[0].0
    var hi = cur[0].0
    for p in cur:
        lo min= p.0
        hi max= p.0
    var span = hi - lo
    var shift = 0
    while span > 0:
        var count = [0 for _ in 0..257]
        for p in cur: count[((p.0 - lo) >> shift) % 256 + 1] += 1
        for d in 0..256: count[d + 1] += count[d]
        var out = cur.clone()
        for p in cur:
            let d = ((p.0 - lo) >> shift) % 256
            out[count[d]] = p
            count[d] += 1
        cur = out
        shift += 8
        span >>= 8
    for i in 0..n: self[i] = cur[i].1

/// Sorts in place by `by(x)`, a position of the domain `K`, keeping equal keys in
/// their order: one count per position of `K`, then each element goes to its slot.
///
/// ```cp
/// domain K = 0..2
/// var a = [(k: K(1), s: "b"), (k: K(0), s: "a"), (k: K(1), s: "c")]
/// a.counting_sort(by: p => p.k)
/// [p.s for p in a] == ["a", "b", "c"]
/// ```
///
/// cost: \(O(n + \lvert K \rvert)\), \(O(n + \lvert K \rvert)\) memory
/// empty: `domain K = 0..2; var a: List<(k: K, s: Str)> = []; a.counting_sort(by: p => p.k); a == []`
/// ties: `domain K = 0..2; var a = [(k: K(1), s: "b"), (k: K(1), s: "a")]; a.counting_sort(by: p => p.k); [p.s for p in a] == ["b", "a"]`
/// traps: never
pub fn counting_sort<S: IndexedMut<D, T>, D: Domain, T, K>(mut self: S, by: T -> K):
    let n = self.len
    var start: [K]Int = 0
    for i in 0..n: start[by(self.nth(i))] += 1
    var acc = 0
    for k in K:
        let c = start[k]
        start[k] = acc
        acc += c
    var pos: List<Int> = []
    for i in 0..n:
        let k = by(self.nth(i))
        pos.push(start[k])
        start[k] += 1
    for i in 0..n:
        while pos[i] != i:
            let j = pos[i]
            self.swap_idx(i, j)
            pos.swap(i, j)

/// \(O(n)\) expected, \(O(n)\) memory. Whether `self` and `b` hold the same multiset of elements; `p.is_permutation(p.keys)` asks whether `p` is a permutation of its own keys.
///
/// cost: \(O(n)\) expected, \(O(n)\) memory
/// empty: `let e: List<Int> = []; e.is_permutation(e) == true`
/// traps: never
/// example: `[1, 2, 2].is_permutation([2, 1, 2]) == true`
/// example: `[2, 0, 1].is_permutation(0..3) == true`
pub fn is_permutation<A: Seq<T>, B: Seq<T>, T: Hash + Eq>(self: A, b: B) -> Bool:
    var left = HashMap<T, Int>()
    var n = 0
    for x in self:
        let c = left.get(x)
        left[x] = if c == None: 1 else: c! + 1
        n += 1
    for y in b:
        let c = left.get(y)
        if c == None or c! == 0: return false
        left[y] = c! - 1
        n -= 1
    n == 0

/// \(O(\lvert \mathit{self} \rvert + \lvert \mathit{of} \rvert)\). Whether `self` occurs as a subsequence of `of`.
///
/// cost: \(O(\lvert \mathit{self} \rvert + \lvert \mathit{of} \rvert)\)
/// empty: `"".is_subsequence("abc") == true`
/// traps: never
/// example: `"ace".is_subsequence("abcde") == true`
pub fn is_subsequence<S: Seq<T>, N: Indexed, T>(self: S, of: N) -> Bool:
    var j = 0
    for x in self:
        while j < of.len and of.nth(j) != x: j += 1
        if j == of.len: return false
        j += 1
    true

// --- Permutations, combinatorics, subsequences, and monotone-stack trees ---

/// \(O(n)\), \(O(1)\) memory. Lexicographic rank of permutation `p` in `0..|V|!` (`|V| <= 20`).
///
/// cost: \(O(n)\), \(O(1)\) memory
/// empty: `domain V = 0..0; let p: [V]V = []; perm_rank(p) == 0`
/// traps: when `|V| > 20` or `p` is not a permutation
/// example: `perm_rank([1, 0, 2]) == 2`
/// trap: `perm_rank([0, 0, 1])`
pub fn perm_rank<V>(self: [V]V) -> Int:
    let n = V.count
    var (seen, rank, fact) = (0, 0, 1)
    for k in (0..n).rev:
        let v = self[V.nth(k)].ord
        assert not seen.bit(v), "not a permutation: {self[V.nth(k)]} appears twice"
        rank += popcount(seen & ((1 << v) - 1)) * fact
        fact *= n - k
        seen |= 1 << v
    rank

/// \(O(n \log n)\), \(O(1)\) memory beyond the result. The `rank`-th lexicographic permutation of `V` (`0 <= rank < |V|!`).
///
/// cost: \(O(n \log n)\), \(O(1)\) memory beyond the result
/// empty: `domain V = 0..0; perm_unrank<V>(0) == []`
/// order: keyed by `V`
/// traps: when `rank` is not in `0..|V|!`
/// example: `domain V = 0..3; perm_unrank<V>(2) == [1, 0, 2]`
pub fn perm_unrank<V>(rank: Int) -> [V]V:
    let n = V.count
    var fact = 1
    for k in 1..n: fact *= k
    let total = if n == 0: 1 else: fact * n
    assert rank in 0..total, "`rank` is not in 0..|V|!"
    var (r, free) = (rank, (1 << n) - 1)
    var out: [V]V = [V] v => v
    for k in 0..n:
        let need = r / fact + 1
        r %= fact
        if k + 1 < n: fact /= n - 1 - k
        let v = (0..n).first_true(x => popcount(free & ((2 << x) - 1)) >= need)!
        out[V.nth(k)] = V.nth(v)
        free ^= 1 << v
    out

@test
fn perm_rank_and_unrank_round_trip_at_twenty():
    domain V = 0..20
    let last: [V]V = [V] v => V.nth(19 - v.ord)
    assert last.perm_rank() == 2432902008176639999
    assert perm_unrank<V>(2432902008176639999) == last
    for r in [0, 1, 7, 123456789, 2432902008176639998]:
        assert perm_unrank<V>(r).perm_rank() == r

/// \(O(n)\), \(O(n)\) memory. Sign/parity of permutation `p` (`+1` even, `-1` odd).
///
/// cost: \(O(n)\), \(O(n)\) memory
/// empty: `domain V = 0..0; let p: [V]V = []; perm_sign(p) == 1`
/// traps: when `p` is not a permutation
/// example: `perm_sign([1, 0, 2]) == -1`
/// trap: `perm_sign([0, 0, 1])`
pub fn perm_sign<V>(self: [V]V) -> Int:
    var seen: [V]Bool = false
    var sign = 1
    for v in V:
        if seen[v]: continue
        var (u, len) = (v, 0)
        while not seen[u]:
            seen[u] = true
            u = self[u]
            len += 1
        assert u == v, "not a permutation: {u} is reached twice"
        if len % 2 == 0: sign = -sign
    sign

/// \(O(n)\), \(O(n)\) memory. Order of permutation `p` (LCM of disjoint cycle lengths).
///
/// cost: \(O(n)\), \(O(n)\) memory
/// empty: `domain V = 0..0; let p: [V]V = []; perm_order(p) == 1`
/// traps: never; the order can overflow `Int` past about |V| = 400
/// example: `perm_order([1, 0, 3, 4, 2]) == 6`
pub fn perm_order<V>(self: [V]V) -> Int:
    var seen: [V]Bool = false
    var ord = 1
    for v in V:
        if seen[v]: continue
        var (u, len) = (v, 0)
        while not seen[u]:
            seen[u] = true
            u = self[u]
            len += 1
        var (a, b) = (ord, len)
        while b != 0: (a, b) = (b, a % b)
        ord = ord / a * len
    ord

/// \(O(\lvert I \rvert + \lvert V \rvert)\), \(O(\lvert I \rvert + \lvert V \rvert)\) memory. Inverse of partial injection `self: [I]?V -> [V]?I` (traps on duplicate non-`None` images).
///
/// cost: \(O(\lvert I \rvert + \lvert V \rvert)\), \(O(\lvert I \rvert + \lvert V \rvert)\) memory
/// empty: `domain I = 0..0; domain V = 0..2; let p: [I]?V = []; p.partial_inverse() == [None, None]`
/// order: keyed by `V`
/// traps: when two keys map to the same value
/// example: `domain I = 0..3; let p: [I]?I = [Some(2), None, Some(0)]; p.partial_inverse() == [Some(2), None, Some(0)]`
pub fn partial_inverse<I, V>(self: [I]?V) -> [V]?I:
    var out: [V]?I = None
    for i in I:
        if let v = self[i]:
            assert out[v] == None, "two positions map to the same value"
            out[v] = i
    out

/// \(O(k)\) per combination. All `k`-element combinations in lexicographic index order (`k = 2` is unordered pairs).
///
/// cost: \(O(k)\) per combination
/// empty: `let e: List<Int> = []; e.combinations(0).collect() == [[]] and e.combinations(1).count() == 0`
/// order: lexicographic by position
/// traps: when `k < 0` or `k >= 2^31`
/// example: `[1, 2, 3].combinations(2).collect() == [[1, 2], [1, 3], [2, 3]]`
pub fn combinations<S: Indexed<D, T>, D: Domain, T>(self: S, k: Len) -> yields List<T>:
    let n = self.len
    if k > n: return
    var idx = (0..k).collect()
    while true:
        yield [self.nth(i) for i in idx]
        var i = k - 1
        while i >= 0 and idx[i] == n - k + i: i -= 1
        if i < 0: return
        idx[i] += 1
        for j in i + 1..k: idx[j] = idx[j - 1] + 1

/// \(O(1)\) per pair. Zero-alloc unordered index pairs `(i, j)` with `i < j` over `self: Range<D>`.
///
/// cost: \(O(1)\) per pair
/// empty: `(0..0).index_pairs().count() == 0`
/// order: by the first key, then the second
/// traps: never
/// example: `(0..3).index_pairs().collect() == [(0, 1), (0, 2), (1, 2)]`
pub fn index_pairs<D>(self: Range<D>) -> yields (D, D):
    for i in 0..self.len:
        for j in i + 1..self.len: yield (self.nth(i), self.nth(j))

/// \(O(k)\) per permutation. All `k`-permutations of `self` (default `k = self.len`).
/// Equal elements give repeated permutations; dedup the input's arrangements with `next_permutation`.
///
/// cost: \(O(k)\) per permutation
/// empty: `let e: List<Int> = []; e.permutations().collect() == [[]]`
/// ties: `[1, 1].permutations().collect() == [[1, 1], [1, 1]]`
/// order: lexicographic by position
/// traps: when `k` is not in `0..=self.len`
/// example: `[1, 2].permutations().collect() == [[1, 2], [2, 1]]`
pub fn permutations<S: Indexed<D, T>, D: Domain, T>(self: S, k: ?Int = None) -> yields List<T>:
    let n = self.len
    let m = k ?? n
    let _ = [0 for _ in 0..n + 1][m]
    var used = [false for _ in 0..n]
    var pick: List<Int> = []
    var next = 0
    while true:
        if pick.len == m:
            yield [self.nth(i) for i in pick]
            if m == 0: return
            next = pick.pop()! + 1
            used[next - 1] = false
        while next < n and used[next]: next += 1
        if next < n:
            used[next] = true
            pick.push(next)
            next = 0
        elif pick.len == 0: return
        else:
            next = pick.pop()! + 1
            used[next - 1] = false

/// \(O(1)\). Steerable item yielded by `subsequences` (decision 126, W6).
pub struct SubseqStep<T>(pub items: List<T>, var pruned: Bool = false):
    /// \(O(1)\). Skip extending this prefix in the `subsequences` search tree.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2].subsequences().filter(s => s.items.len <= 1 or { s.prune(); false }).count() == 3`
    pub fn prune(mut self):
        pruned = true

/// \(O(k)\) per subsequence. All subsequences of `self` in lex order; supports `step.prune()` (decision 126, W6).
///
/// cost: \(O(k)\) per subsequence
/// empty: `let e: List<Int> = []; e.subsequences().map(s => s.items).collect() == [[]]`
/// order: lexicographic by position, the empty one first; `prune` skips every extension of the current one
/// traps: never
/// example: `[1, 2].subsequences().map(s => s.items).collect() == [[], [1], [1, 2], [2]]`
@lint(subset_masks_by_hand, from: "for mask in 0..(1 << n):\n    let s = sum(self[i] for i in 0..n if mask.bit(i))\n    body", advice: "self.subsequences()", holes: "n, body")
@lint(subset_masks_by_hand, from: "for mask in 0..(1 << n):\n    let s = sum(self[i] for i in 0..n if (mask >> i) & 1 == 1)\n    body", advice: "self.subsequences()", holes: "n, body")
@lint(subset_masks_by_hand, from: "for mask in 0..(1 << n):\n    var s = 0\n    for i in 0..n:\n        if mask.bit(i):\n            s += self[i]\n    body", advice: "self.subsequences()", holes: "n, body")
@lint(subset_masks_by_hand, from: "for mask in 0..(1 << n):\n    var s = 0\n    for i in 0..n:\n        if (mask >> i) & 1 == 1:\n            s += self[i]\n    body", advice: "self.subsequences()", holes: "n, body")
pub fn subsequences<S: Indexed<D, T>, D: Domain, T>(self: S) -> yields SubseqStep<T>:
    let n = self.len
    var pick: List<Int> = []
    while true:
        let step = SubseqStep<T>([self.nth(i) for i in pick], false)
        yield step
        if not step.pruned:
            let next = if pick.len > 0: pick.last! + 1 else: 0
            if next < n:
                pick.push(next)
                continue
        while pick.len > 0 and pick.last! + 1 >= n:
            pick.pop()
        if pick.len == 0: return
        pick[pick.len - 1] += 1

/// \(O(n \log n)\), \(O(n)\) memory. The positions of one longest increasing subsequence by `by`;
/// `strict: false` allows equal neighbours. Longest non-increasing is `by: desc`-style `x => -x`
/// with `strict: false`.
///
/// Among longest subsequences it gives the one whose positions are lexicographically greatest from the end (the patience-sorting choice).
///
/// cost: \(O(n \log n)\), \(O(n)\) memory
/// empty: `let e: List<Int> = []; e.lis() == []`
/// ties: `[2, 1, 3].lis() == [1, 2]`
/// order: increasing positions of `self`
/// traps: never
/// example: `[3, 1, 2, 2, 5].lis() == [1, 2, 4]`
/// example: `[3, 1, 2, 2, 5].lis(strict: false).len == 4`
pub fn lis<D, T, K: Ord>(self: [D]T, strict: Bool = true, by: T -> K = x => x) -> List<D>:
    var tails: List<D> = []
    var prev: [D]?D = None
    for i in D:
        let k = by(self[i])
        var (lo, hi) = (0, tails.len)
        while lo < hi:
            let m = lo + (hi - lo) / 2
            let t = by(self[tails[m]])
            if (if strict: t < k else: t <= k): lo = m + 1 else: hi = m
        if strict and lo < tails.len and by(self[tails[lo]]) == k: continue
        if lo > 0: prev[i] = tails[lo - 1]
        if lo == tails.len: tails.push(i) else: tails[lo] = i
    var out: List<D> = []
    var cur: ?D = if tails.len > 0: Some(tails.last!) else: None
    while let c = cur:
        out.push(c)
        cur = prev[c]
    let res: List<D> = [out[i] for i in (0..out.len).rev]
    res

/// \(O(\lvert a \rvert \cdot \lvert b \rvert)\), \(O(\lvert a \rvert \cdot \lvert b \rvert)\) memory. Longest common subsequence of `a` and `b` as matched `(i, j)` index pairs.
///
/// Among longest it gives the one the standard backward walk finds, preferring to step in `a` first.
///
/// cost: \(O(\lvert a \rvert \cdot \lvert b \rvert)\), \(O(\lvert a \rvert \cdot \lvert b \rvert)\) memory
/// empty: `lcs("", "abc") == []`
/// ties: `lcs("ab", "ba") == [(0, 1)]`
/// order: increasing `(i, j)`
/// traps: never
/// example: `lcs("abc", "acb").len == 2`
pub fn lcs<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<(Int, Int)>:
    let (n, m) = (a.len, b.len)
    var f: [n + 1, m + 1]Int = 0
    for i in 0..n:
        for j in 0..m:
            f[i + 1, j + 1] = if a.nth(i) == b.nth(j): f[i, j] + 1 elif f[i, j + 1] < f[i + 1, j]: f[i + 1, j] else: f[i, j + 1]
    var out: List<(Int, Int)> = []
    var (i, j) = (n, m)
    while i > 0 and j > 0:
        if a.nth(i - 1) == b.nth(j - 1):
            out.push((i - 1, j - 1))
            i -= 1
            j -= 1
        elif f[i - 1, j] == f[i, j]: i -= 1
        else: j -= 1
    reverse(mut out)
    out

/// \(O(n)\), \(O(n)\) memory. Min-heap (or max-heap when `min_root: false`) Cartesian tree in one monotone-stack pass.
///
/// cost: \(O(n)\), \(O(n)\) memory
/// empty: `let e: List<Int> = []; let t = e.cartesian_tree(); t.root == None and t.left == [] and t.parent == []`
/// ties: `[1, 1].cartesian_tree().root == Some(0)`
/// order: arrays keyed by `D`
/// traps: never
/// example: `[3, 1, 2].cartesian_tree().root == Some(1)`
pub fn cartesian_tree<D, T: Ord>(self: [D]T, min_root: Bool = true) -> (
    root: ?D,
    left: [D]?D,
    right: [D]?D,
    parent: [D]?D,
):
    var left: [D]?D = None
    var right: [D]?D = None
    var parent: [D]?D = None
    var st: List<D> = []
    for i in D:
        var last: ?D = None
        while st.len > 0 and (
            if min_root: self[st.last!] > self[i] else: self[st.last!] < self[i]
        ):
            last = st.pop()
        if let l = last:
            left[i] = l
            parent[l] = i
        if st.len > 0:
            let t = st.last!
            right[t] = i
            parent[i] = t
        st.push(i)
    (root: if st.len > 0: Some(st[0]) else: None, left:, right:, parent:)

// --- Incremental / online sequence structures (with mark/undo_to rollback for tree DFS) ---

/// \(O(1)\) construction; \(O(\log n)\) push, \(O(1)\) amortized rollback. Online patience-sorting LIS tracker.
pub struct OnlineLis<T>(strict: Bool = true):
    tails: List<T> = []
    tail_at: List<Int> = []
    vals: List<T> = []
    pub prev: List<Int> = []
    pub ends: List<Int> = []
    // Per push: the tail slot it wrote, and what it replaced (`-1` when it appended).
    log_slot: List<Int> = []
    log_val: List<T> = []
    log_at: List<Int> = []

    /// \(O(\log n)\). Append `x` and return the length of the longest increasing subsequence ending at `x`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); t.push(5); t.push(1); t.push(7) == 2`
    pub fn push(mut self, x: T) -> Int where T: Ord:
        let lo = (0..tails.len).first_true(m => not (tails[m] < x or (not strict and tails[m] == x))) ?? tails.len
        prev.push(if lo > 0: tail_at[lo - 1] else: -1)
        ends.push(lo + 1)
        vals.push(x)
        log_slot.push(lo)
        if lo == tails.len:
            log_val.push(x)
            log_at.push(-1)
            tails.push(x)
            tail_at.push(vals.len - 1)
        else:
            log_val.push(tails[lo])
            log_at.push(tail_at[lo])
            tails[lo] = x
            tail_at[lo] = vals.len - 1
        lo + 1
    /// \(O(1)\). Current global LIS length.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); t.push(1); t.push(3); t.len == 2`
    @get pub fn len(self) -> Int = tails.len
    /// \(O(1)\). Whether no elements have been pushed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `OnlineLis<Int>().is_empty == true`
    @get pub fn is_empty(self) -> Bool = vals.len == 0

    /// \(O(\mathit{len})\). Visit one longest increasing subsequence as `(push index, value)`
    /// pairs: the one `reconstruct` returns, with its values.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `OnlineLis<Int>().items.collect() == []`
    /// ties: `var t = OnlineLis<Int>(); for x in [1, 3, 2]: t.push(x); t.items.collect() == [(0, 1), (2, 2)]`
    /// order: increasing push index
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); for x in [3, 1, 2]: t.push(x); t.items.collect() == [(1, 1), (2, 2)]`
    @get pub fn items(self) -> yields (Int, T):
        for i in self.reconstruct(): yield (i, vals[i])

    /// \(O(1)\). Reset all state.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); t.push(1); t.clear(); t.len == 0`
    pub fn clear(mut self):
        tails.clear()
        tail_at.clear()
        vals.clear()
        prev.clear()
        ends.clear()
        log_slot.clear()
        log_val.clear()
        log_at.clear()
    /// \(O(n)\), \(O(n)\) memory. Reconstruct one global LIS as 0-based push indices.
    ///
    /// cost: \(O(n)\), \(O(n)\) memory
    /// empty: `OnlineLis<Int>().reconstruct() == []`
    /// ties: `var t = OnlineLis<Int>(); for x in [1, 3, 2]: t.push(x); t.reconstruct() == [0, 2]`
    /// order: increasing push indices
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); for x in [3, 1, 2]: t.push(x); t.reconstruct() == [1, 2]`
    pub fn reconstruct(self) -> List<Int>:
        if tails.len == 0: return []
        var out = [0 for _ in 0..tails.len]
        var i = tail_at[tails.len - 1]
        var k = tails.len - 1
        while i >= 0:
            out[k] = i
            i = prev[i]
            k -= 1
        out
    /// \(O(1)\). Save a rollback checkpoint for DFS-on-tree LIS.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); let m = t.mark(); t.push(1); t.undo_to(m); t.len == 0`
    pub fn mark(self) -> Int = vals.len
    /// \(O(1)\) amortized. Roll back all `push`es since checkpoint `m`.
    ///
    /// cost: \(O(1)\) amortized per undone push
    /// traps: when `m` is not a live mark of this tracker
    /// example: `var t = OnlineLis<Int>(); t.push(2); let m = t.mark(); t.push(5); t.undo_to(m); t.len == 1`
    pub fn undo_to(mut self, m: Int):
        assert 0 <= m <= vals.len  // `m` is a live mark
        while vals.len > m:
            let slot = log_slot.pop()!
            let v = log_val.pop()!
            let at = log_at.pop()!
            if at < 0:
                tails.pop()
                tail_at.pop()
            else:
                tails[slot] = v
                tail_at[slot] = at
            vals.pop()
            prev.pop()
            ends.pop()

/// \(O(1)\) construction; \(O(1)\) amortized push and rollback. Online monotone stack returning nearest dominating index.
pub struct MonoStack<D, T>(keep_smaller: Bool = true, strict: Bool = true):
    idx: List<D> = []
    vals: List<T> = []
    // Per push: how many entries it popped; the popped entries, oldest pop first.
    popped: List<Int> = []
    out_idx: List<D> = []
    out_val: List<T> = []

    fn pops(self, top: T, x: T) -> Bool where T: Ord:
        if top == x: return strict
        if keep_smaller: top < x else: top > x

    /// \(O(1)\) amortized. Push `(i, x)`, pop dominated entries, and return the nearest dominating index (or `None`).
    ///
    /// cost: \(O(1)\) amortized
    /// ties: `var s = MonoStack<Int, Int>(); s.push(0, 5); var t = MonoStack<Int, Int>(strict: false); t.push(0, 5); s.push(1, 5) == None and t.push(1, 5) == Some(0)`
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); s.push(1, 3) == Some(0)`
    pub fn push(mut self, i: D, x: T) -> ?D:
        var n = 0
        while vals.len > 0 and self.pops(vals.last!, x):
            out_idx.push(idx.pop()!)
            out_val.push(vals.pop()!)
            n += 1
        popped.push(n)
        let below = if idx.len > 0: Some(idx.last!) else: None
        idx.push(i)
        vals.push(x)
        below
    /// \(O(1)\). Current top `(index, val)`, or `None` when empty.
    /// Ties don't apply: the stack has one top.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(3, 8); s.top == Some((index: 3, val: 8))`
    /// ties: `var s = MonoStack<Int, Int>(strict: false); s.push(0, 5); s.push(1, 5); s.top == Some((index: 1, val: 5))`
    @get pub fn top(self) -> ?(index: D, val: T):
        if idx.len == 0: None else: Some((index: idx.last!, val: vals.last!))
    /// \(O(1)\). Current stack depth.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); s.len == 1`
    @get pub fn len(self) -> Int = idx.len
    /// \(O(1)\). Whether the stack is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MonoStack<Int, Int>().is_empty == true`
    @get pub fn is_empty(self) -> Bool = idx.len == 0

    /// \(O(\mathit{len})\). Visit the kept `(index, val)` entries, bottom of the stack first.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `MonoStack<Int, Int>().items.collect() == []`
    /// order: push order, bottom to top
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); s.push(1, 3); s.push(2, 4); s.items.collect() == [(0, 5), (2, 4)]`
    @get pub fn items(self) -> yields (D, T):
        for j in 0..idx.len: yield (idx[j], vals[j])

    /// \(O(1)\). Empty the stack and rollback log.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 1); s.clear(); s.is_empty`
    pub fn clear(mut self):
        idx.clear()
        vals.clear()
        popped.clear()
        out_idx.clear()
        out_val.clear()
    /// \(O(1)\). Save a rollback checkpoint for DFS-on-tree monotone stacks.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); let m = s.mark(); s.push(0, 1); s.undo_to(m); s.is_empty`
    pub fn mark(self) -> Int = popped.len
    /// \(O(1)\) amortized. Roll back all `push`es since checkpoint `m`.
    ///
    /// cost: \(O(1)\) amortized per undone push
    /// traps: when `m` is not a live mark of this stack
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); let m = s.mark(); s.push(1, 9); s.undo_to(m); s.len == 1`
    pub fn undo_to(mut self, m: Int):
        assert 0 <= m <= popped.len  // `m` is a live mark
        while popped.len > m:
            idx.pop()
            vals.pop()
            var n = popped.pop()!
            while n > 0:
                idx.push(out_idx.pop()!)
                vals.push(out_val.pop()!)
                n -= 1

@test
fn sorted_lookup_and_linear_search():
    // Sequence first_ge / last_le return indices (?Int on List, ?D on [D]T)
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_ge(3) == 1 and xs.first_ge(4) == 3 and xs.first_ge(11) == None
    assert xs.last_le(3) == 2 and xs.last_le(0) == None and xs.last_le(99) == 4
    assert xs.find(7) == 3 and xs.find(8) == None
    domain V = 1..=5
    let a = [V] v => xs[v.ord]
    assert a.first_ge(4) == V(4) and a.last_le(3) == V(3) and a.find(10) == V(5)
    assert first(x for x in xs if x > 3) == 7 and first(x for x in xs if x > 10) == None
    let words = ["pear", "fig", "banana", "kiwi"]
    assert words.min(by: w => w.len) == "fig" and words.max(by: w => w.len) == "banana"
    let nothing: List<Str> = []
    assert nothing.min(by: w => w.len) == None
    domain D = 0..5
    assert [w.len for w in D.windows(3)] == [3, 3, 3] and D.windows(3).collect()[1] == (D(1)..=D(3))

@test
fn reorder_and_permutations_in_place():
    var a = [1, 1, 2, 3, 3, 3, 1]
    dedup(mut a)
    assert a == [1, 2, 3, 1]
    rotate_left(mut a, 1)
    assert a == [2, 3, 1, 1]
    rotate_left(mut a, -1)
    assert a == [1, 2, 3, 1]
    // Negative k rotates right (Euclidean modulo)
    var rot = [1, 2, 3, 4, 5]
    rotate_left(mut rot, -2)
    assert rot == [4, 5, 1, 2, 3]
    rotate_left(mut rot, -7)
    assert rot == [2, 3, 4, 5, 1]
    reverse(mut a)
    assert a == [1, 3, 2, 1]
    // next_permutation and prev_permutation are exact mirrors
    var p = [1, 2, 2]
    var fwd = [p.clone()]
    while next_permutation(mut p): fwd.push(p.clone())
    assert fwd == [[1, 2, 2], [2, 1, 2], [2, 2, 1]] and p == [1, 2, 2]
    var q = [2, 2, 1]
    var bwd = [q.clone()]
    while prev_permutation(mut q): bwd.push(q.clone())
    assert bwd == [[2, 2, 1], [2, 1, 2], [1, 2, 2]] and q == [2, 2, 1]
    // 4-element full round-trip forward and backward
    var perm = [1, 2, 3, 4]
    var up = [perm.clone()]
    while next_permutation(mut perm): up.push(perm.clone())
    assert up.len == 24 and perm == [1, 2, 3, 4]
    var perm_down = [4, 3, 2, 1]
    var down = [perm_down.clone()]
    while prev_permutation(mut perm_down): down.push(perm_down.clone())
    assert down.len == 24 and perm_down == [4, 3, 2, 1]
    assert down == up.rev
    // Edge cases: empty and singleton
    var empty: List<Int> = []
    assert not next_permutation(mut empty) and not prev_permutation(mut empty)
    var one = [42]
    assert not next_permutation(mut one) and not prev_permutation(mut one) and one == [42]
    assert [(r[r.keys.first!], r.len) for r in "aabccc".runs()] == [('a', 2), ('b', 1), ('c', 3)]

@test
fn compress_gives_the_values_their_own_domain():
    let ys = [40, -7, 40, 1_000_000_000, 3]
    let (domain C, xs) = compress(ys)
    assert xs.len == 4 and C.count == 4
    assert [xs[i] for i in C] == [-7, 3, 40, 1_000_000_000]
    var cnt: [C]Int = 0
    for y in ys: cnt[xs.index_of(y)] += 1
    assert [cnt[i] for i in C] == [1, 1, 2, 1]
    assert xs.first_ge(4) == xs.index_of(40) and xs.first_ge(2_000_000_000) == None
    assert xs.last_le(4) == xs.index_of(3) and xs.last_le(-100) == None
    let (domain O, other) = compress([5, 6])
    assert O.count == 2 and other.len == 2 and C.count == 4
    // Compressed range slicing
    let blk = C.first!..=C.nth(2)
    assert xs.slice(blk) == [-7, 3, 40]
    assert xs.slice(C.nth(1)..C.nth(1)) == []

@test
fn min_max_by_tie_breaking():
    let items = [
        (10, "first_min"),
        (5, "a"),
        (5, "b"),
        (20, "first_max"),
        (20, "second_max"),
        (5, "c"),
    ]
    assert items.min(by: p => p.0) == Some((5, "a"))
    assert items.max(by: p => p.0) == Some((20, "first_max"))
    assert [("x", 1), ("y", 1), ("z", 1)].min(by: p => p.1) == Some(("x", 1))
    assert [("x", 1), ("y", 1), ("z", 1)].max(by: p => p.1) == Some(("x", 1))
    let empty: List<(Int, Str)> = []
    assert empty.min(by: p => p.0) == None and empty.max(by: p => p.0) == None

@test
fn range_bounds_and_keyed_sorted_lookup():
    domain D = 1..=10
    // Range.l and Range.r boundary cuts even when empty
    let empty = D(4)..D(4)
    assert empty.l == D(4).l and empty.r == D(4).l and empty.l == empty.r
    let seg = D(2)..=D(5)
    assert seg.l == D(2).l and seg.r == D(5).r
    // first_ge / last_le with by: return ?Int
    let pairs = [(1, "a"), (3, "b"), (3, "c"), (7, "d")]
    assert pairs.first_ge(3, by: p => p.0) == 1
    assert pairs.first_ge(4, by: p => p.0) == 3
    assert pairs.first_ge(9, by: p => p.0) == None
    assert pairs.last_le(3, by: p => p.0) == 2
    assert pairs.last_le(0, by: p => p.0) == None

@test
fn list_insert_remove_take_and_swap_at():
    domain I = 0..4
    var a = [I] i => i.ord * 10
    a.swap_at(I(1), I(3))
    a.swap_at(I(2), I(2))
    assert [a[i] for i in I] == [0, 30, 20, 10]
    let p = [1, 2, 3]
    assert p.resized(5, 0) == [1, 2, 3, 0, 0] and p.resized(2, 0) == [1, 2]
    assert p.resized(0, 9).is_empty and p == [1, 2, 3]
    for _ in 0..30:
        let xs = [rand(-9, 9) for _ in 0..rand(0, 8)]
        let n = rand(0, 10)
        let r = xs.resized(n, 7)
        assert r.len == n
        for i in 0..n: assert r[i] == (if i < xs.len: xs[i] else: 7)

@test
fn map_entry_remove_and_four_lookups():
    // Ordered Map 4-lookups return ?(K, V)
    var m = Map<Int, Str>()
    m[10] = "ten"
    m[20] = "twenty"
    m[30] = "thirty"
    assert m.first_ge(20) == (20, "twenty") and m.first_ge(21) == (
        30,
        "thirty",
    ) and m.first_ge(31) == None
    assert m.first_gt(20) == (30, "thirty") and m.first_gt(10) == (
        20,
        "twenty",
    ) and m.first_gt(30) == None
    assert m.last_le(20) == (20, "twenty") and m.last_le(19) == (10, "ten") and m.last_le(5) == None
    assert m.last_lt(20) == (10, "ten") and m.last_lt(30) == (
        20,
        "twenty",
    ) and m.last_lt(10) == None
    // Map.remove returns ?V (the removed value or None)
    assert m.remove(20) == "twenty"
    assert m.remove(20) == None
    assert m.get(20) == None and m.len == 2
    // Map compound assignment auto-inserts from operator identity; plain assign inserts/overwrites
    var cnt = Map<Int, Int>()
    cnt[5] += 3
    cnt[5] += 2
    cnt[8] min= 4
    cnt[8] min= 9
    cnt[9] max= -7
    assert cnt[5] == 5 and cnt[8] == 4 and cnt[9] == -7
    assert cnt.remove(5) == 5 and cnt.remove(5) == None
    // Differential check against brute-force scans on random maps
    for _ in 0..25:
        var rm = Map<Int, Int>()
        var ks: List<Int> = []
        for _ in 0..rand(0, 12):
            let k = rand(-10, 10)
            let v = rand(0, 99)
            rm[k] = v
            if not ks.contains(k): ks.push(k)
        sort(mut ks)
        for q in -12..=12:
            let want_ge = first((k, rm[k]) for k in ks if k >= q)
            let want_gt = first((k, rm[k]) for k in ks if k > q)
            let want_le = [(k, rm[k]) for k in ks if k <= q].max(by: p => p.0)
            let want_lt = [(k, rm[k]) for k in ks if k < q].max(by: p => p.0)
            assert rm.first_ge(q) == want_ge
            assert rm.first_gt(q) == want_gt
            assert rm.last_le(q) == want_le
            assert rm.last_lt(q) == want_lt

@test
fn deduped_and_partition_leave_the_list_alone():
    let xs = [3, 1, 3, 2, 1]
    assert sorted(xs).deduped() == [1, 2, 3]
    assert xs.deduped() == xs
    let (small, big) = xs.partition(x => x < 3)
    assert small == [1, 2, 1] and big == [3, 3]
    assert xs == [3, 1, 3, 2, 1]
    for _ in 0..50:
        let ys = [rand(0, 5) for _ in 0..rand(0, 12)]
        let (even, odd) = ys.partition(y => y % 2 == 0)
        assert even.len + odd.len == ys.len and even.all(y => y % 2 == 0) and odd.all(y => y % 2 == 1)

@test
fn get_or_insert_stores_only_what_is_missing():
    var ids = Map<Str, Int>()
    assert ids.get_or_insert("a", ids.len) == 0 and ids.get_or_insert("b", ids.len) == 1
    assert ids.get_or_insert("a", 99) == 0 and ids.len == 2

@test
fn update_starts_from_init_and_then_from_the_value():
    var best = Map<Str, Int>()
    best.update("a", 0, c => c + 1)
    best.update("a", 0, c => c + 1)
    best.update("b", 10, c => if c < 4: c else: 4)
    assert best["a"] == 2 and best["b"] == 4 and best.len == 2

@test
fn runs_with_by_groups_neighbours_with_the_same_key():
    let rs = [(r[r.keys.first!][0], r.len) for r in ["ab", "ac", "b", "ad"].runs(by: s => s[0])]
    assert rs == [('a', 2), ('b', 1), ('a', 1)]
    let none_yet: List<Int> = []
    assert none_yet.runs(by: x => x).len == 0

@test
fn windows_and_zip_walk_in_step():
    assert [w.to_list() for w in [1, 2, 3, 4].windows(2)] == [[1, 2], [2, 3], [3, 4]]
    assert [w.to_list() for w in [1, 2].windows(2)] == [[1, 2]] and [1, 2].windows(3).count() == 0
    assert zip([1, 2, 3], ["a", "b"]).collect() == [(1, "a"), (2, "b")] and zip([1], List<Int>()).len == 0

@test
fn adjacent_and_with_prev_yield_neighbours():
    assert [10, 20, 30].adjacent.collect() == [(10, 20), (20, 30)]
    assert [10, 20, 30].pairs().collect() == [(10, 20), (10, 30), (20, 30)]
    assert [7].adjacent.count() == 0
    let empty: List<Int> = []
    assert empty.adjacent.count() == 0 and empty.pairs().count() == 0
    assert [(p ?? -1, x) for (p, x) in [10, 20, 30].with_prev()] == [(-1, 10), (10, 20), (20, 30)]
    assert [(p ?? -1, x) for (p, x) in [7].with_prev()] == [(-1, 7)]
    assert empty.with_prev().collect().len == 0

@test
fn cyclic_pairs_close_the_ring():
    assert ([1, 2, 3].cyclic_pairs()).collect() == [(1, 2), (2, 3), (3, 1)]
    assert ([7].cyclic_pairs()).collect() == [(7, 7)]
    let empty: List<Int> = []
    assert empty.cyclic_pairs().collect().len == 0

/// The index of the first largest value (optionally under key `by:`), or `None` when empty.
///
/// `[3, 9, 2, 9].argmax()` is `Some(1)` and `words.argmax(by: w => w.len)` is the index of the
/// longest word. It is an index of `self`, so `a[a.argmax()!]` is the largest value of a
/// `List<T>`, a `[1..=n]T`, a `[V]T` or a filtered view of one alike. \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.argmax() == None`
/// ties: `[(9, 'a'), (9, 'b')].argmax(by: .0) == Some(0)`
/// traps: never
/// example: `[3, 9, 2, 9].argmax() == Some(1)`
/// example: `[3, 9, 2, 9].argmax(ties: Ties.Last) == Some(3)`
pub fn argmax<S: Keyed<D, T>, D, T, K: Ord>(self: S, by: T -> K = x => x, ties: Ties = Ties.First) -> ?D:
    let last = ties == Ties.Last
    var best: ?D = None
    var high: ?K = None
    for (i, x) in self.items:
        let k = by(x)
        if high == None or high! < k or (last and high! == k):
            best = Some(i)
            high = Some(k)
    best

/// The index of the first smallest value (optionally under key `by:`), or `None` when empty.
///
/// cost: \(O(n)\)
/// empty: `let a: List<Int> = []; a.argmin() == None`
/// ties: `[(1, 'a'), (1, 'b')].argmin(by: .0) == Some(0)`
/// traps: never
/// example: `[3, 1, 1].argmin() == Some(1)`
/// example: `[3, 1, 1].argmin(ties: Ties.Last) == Some(2)`
pub fn argmin<S: Keyed<D, T>, D, T, K: Ord>(self: S, by: T -> K = x => x, ties: Ties = Ties.First) -> ?D:
    let last = ties == Ties.Last
    var best: ?D = None
    var low: ?K = None
    for (i, x) in self.items:
        let k = by(x)
        if low == None or k < low! or (last and k == low!):
            best = Some(i)
            low = Some(k)
    best

@test
fn argmax_and_argmin_give_the_first_best_index():
    assert [3, 9, 2, 9].argmax() == Some(1)
    assert [3, 2, 9, 2].argmin() == Some(1)
    let empty: List<Int> = []
    assert empty.argmax() == None and empty.argmin() == None
    for _ in 0..50:
        let a = [rand(0, 5) for _ in 0..rand(1, 8)]
        let hi = a.argmax()!
        for i in 0..a.len: assert a[i] <= a[hi] and (i >= hi or a[i] < a[hi])
        let lo = a.argmin()!
        for i in 0..a.len: assert a[i] >= a[lo] and (i >= lo or a[i] > a[lo])
    let words = ["pear", "fig", "banana", "kiwi", "orange"]
    assert words.argmax(by: w => w.len) == Some(2)
    assert words.argmin(by: w => w.len) == Some(1)

@test
fn prod_and_is_sorted_cover_sequences():
    assert prod([2, 3, 5]) == 30
    let empty_int: List<Int> = []
    assert prod(empty_int) == 1
    assert abs(prod([1.5, 2.0]) - 3.0) < 1e-12
    assert [1, 2, 2, 5].is_sorted() and not [1, 3, 2].is_sorted()
    assert empty_int.is_sorted() and [42].is_sorted()
    assert ["a", "bb", "ccc"].is_sorted(by: s => s.len)
    assert not ["bb", "a", "ccc"].is_sorted(by: s => s.len)

@test
fn select_merge_sorted_and_rotate_right():
    for _ in 0..50:
        let n = rand(1, 10)
        var a = [rand(-9, 9) for _ in 0..n]
        let want = sorted(a.clone())
        let k = rand(0, n - 1)
        a.select(k)
        assert a.nth(k) == want[k]
        for i in 0..k: assert a[i] <= a[k]
        for i in k + 1..n: assert a[i] >= a[k]
        let x = sorted([rand(0, 9) for _ in 0..rand(0, 6)])
        let y = sorted([rand(0, 9) for _ in 0..rand(0, 6)])
        var both = x.clone()
        for v in y: both.push(v)
        assert merge_sorted(x, y) == sorted(both)
        let r = rand(-12, 12)
        var z = x.clone()
        z.rotate_right(r)
        var back = z.clone()
        back.rotate_left(r)
        assert back == x

@test
fn chunk_by_run_ranges_buckets_and_chunks():
    assert ([1, 2, 3].pairs()).collect() == [(1, 2), (1, 3), (2, 3)]
    assert ([1, 2].cyclic_pairs()).collect() == [(1, 2), (2, 1)]
    domain V = 1..=5
    domain B = 0..3
    let keys = [V] i => B([2, 0, 2, 1, 0][i.ord])
    let b = keys.buckets()
    assert b[B(0)] == [V(2), V(5)] and b[B(1)] == [V(4)] and b[B(2)] == [V(1), V(3)]

@test
fn first_gt_and_last_lt_match_linear_scan():
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_gt(3) == Some(3) and xs.first_gt(10) == None and xs.first_gt(0) == Some(0)
    assert xs.last_lt(3) == Some(0) and xs.last_lt(1) == None and xs.last_lt(99) == Some(4)
    let (domain C, c) = compress([10, 3, 7, 3, 1])
    assert c.contains(7) and not c.contains(5)
    assert c.try_index_of(7) == c.index_of(7) and c.try_index_of(5) == None
    assert c.first_gt(3) == c.index_of(7) and c.first_gt(10) == None
    assert c.last_lt(3) == c.index_of(1) and c.last_lt(1) == None
    // Keyed lookup and domain array lookup
    let pairs = [(1, "a"), (3, "b"), (3, "c"), (7, "d")]
    assert pairs.first_gt(3, by: p => p.0) == Some(3)
    assert pairs.last_lt(3, by: p => p.0) == Some(0)
    domain V = 1..=5
    let va = [V] i => xs[i.ord]
    assert va.first_gt(3) == V(4) and va.last_lt(3) == V(1)
    assert xs[1..4].first_gt(3) == Some(2) and xs[1..4].last_lt(3) == None
    assert xs[0..4].last_lt(3) == Some(0) and xs[0..4].first_gt(7) == None
    assert pairs[1..4].first_gt(3, by: p => p.0) == Some(3)
    assert pairs[0..3].last_lt(3, by: p => p.0) == Some(0)
    let s = "abccd"
    assert s[1..4].first_gt('b') == Some(1) and s[1..4].last_lt('c') == Some(0)
    for _ in 0..50:
        let a = sorted([rand(-5, 5) for _ in 0..rand(0, 12)])
        for q in -6..=6:
            assert a.first_gt(q) == first(i for i in 0..a.len if a[i] > q)
            assert a.last_lt(q) == max(i for i in 0..a.len if a[i] < q)

@test
fn permutations_and_inversions():
    domain I = 1..=5
    let p = [I] i => I([3, 5, 4, 1, 2][i.ord])
    let inv = p.inverse()
    for i in I:
        assert inv[p[i]] == i and p[inv[i]] == i
    domain P = 0..5
    let q = [P] i => I([3, 5, 4, 1, 2][i.ord])
    let inv_q = q.inverse()
    for i in P:
        assert inv_q[q[i]] == i and q[inv_q[q[i]]] == q[i]
    let id = compose(p, inv)
    for i in I: assert id[i] == i
    let cycs = perm_cycles(p)
    assert cycs == [[I(1), I(3), I(4)], [I(2), I(5)]]
    // perm_pow matches repeated compose and handles large k
    let p0 = perm_pow(p, 0)
    for i in I: assert p0[i] == i
    let p2 = compose(p, p)
    let p2_fast = perm_pow(p, 2)
    for i in I: assert p2[i] == p2_fast[i]
    let p6 = perm_pow(p, 6)
    for i in I: assert p6[i] == i
    let p_big = perm_pow(p, 1_000_000_000_004)
    for i in I: assert p_big[i] == p2[i]
    // Inversions against O(n^2) brute force
    assert inversions([1, 2, 3]) == 0 and inversions([3, 2, 1]) == 3
    let empty: List<Int> = []
    assert inversions(empty) == 0 and inversions([5]) == 0
    for _ in 0..60:
        let a = [rand(0, 6) for _ in 0..rand(0, 15)]
        assert inversions(a) == a.pairs().count(pq => pq.0 > pq.1)

@test
fn monotone_stack_and_sliding_extrema():
    for _ in 0..60:
        let n = rand(0, 12)
        let xs = [rand(0, 4) for _ in 0..n]
        let ps = xs.prev_smaller()
        let pn = xs.prev_smaller(strict: false)
        let ns = xs.next_smaller()
        let nn = xs.next_smaller(strict: false)
        for i in 0..n:
            var want_ps: ?Int = None
            var want_pn: ?Int = None
            for j in 0..i:
                if xs[j] < xs[i]: want_ps = Some(j)
                if xs[j] <= xs[i]: want_pn = Some(j)
            var want_ns: ?Int = None
            var want_nn: ?Int = None
            for j in (i + 1..n).rev:
                if xs[j] < xs[i]: want_ns = Some(j)
                if xs[j] <= xs[i]: want_nn = Some(j)
            assert ps[i] == want_ps and pn[i] == want_pn and ns[i] == want_ns and nn[i] == want_nn

@test
fn seq_and_indexed_traits_unify_sequences_and_views():
    let xs = [1, 1, 2, 2, 2]
    assert xs.first == Some(1) and xs.last == Some(2)
    assert xs.is_sorted() and not [3, 1].is_sorted()
    domain V = 1..=5
    let arr: [V]Int = [10, 10, 20, 30, 30]
    assert arr.is_sorted() and arr.first == Some(10) and arr.last == Some(30)
    let r = 1..=5
    assert r.is_sorted() and r.first == Some(1) and r.last == Some(5)
    assert sum(r) == 15 and r.prod() == 120
    let vs = [1, 3, 5, 2, 4, 6]
    assert merge_sorted([1, 3, 5], [2, 4, 6]) == [1, 2, 3, 4, 5, 6]
    assert zip(vs, [10, 10, 20, 30, 30]).collect() == [
        (1, 10),
        (3, 10),
        (5, 20),
        (2, 30),
        (4, 30),
    ]
    assert inversions(vs) == 3 and inversions(arr) == 0 and inversions(r) == 0

@test
fn distinct_with_by_key_keeps_first_seen():
    let pairs = [(1, "a"), (2, "b"), (1, "c"), (3, "d"), (2, "e")]
    assert pairs.distinct(by: p => p.0) == [(1, "a"), (2, "b"), (3, "d")]
    let words = ["ant", "ape", "bat", "arc", "box"]
    assert words.distinct(by: w => w[0]) == ["ant", "bat"]
    let empty: List<(Int, Str)> = []
    assert empty.distinct(by: p => p.0) == []
    domain D = 0..5
    let arr: [D]Int = [10, 21, 30, 41, 52]
    assert arr.distinct(by: x => x % 10) == [10, 21, 52]
    assert "aAbBcA".distinct(by: c => if c >= 'a': c - 'a' else: c - 'A') == ['a', 'b', 'c']

@test
fn scan_prod_and_rotate_method_syntax():
    let xs = [3, 1, 4, 1, 5]
    assert xs.prod() == 60
    assert xs.rotated_left(2) == [4, 1, 5, 3, 1]
    assert xs.rotated_left(-1) == [5, 3, 1, 4, 1]
    assert xs.rotated_right(2) == [1, 5, 3, 1, 4]
    let empty: List<Int> = []
    assert empty.rotated_left(3) == []
    var m = [1, 2, 3, 4]
    m.rotate_left(1)
    assert m == [2, 3, 4, 1]
    m.rotate_right(2)
    assert m == [4, 1, 2, 3]
    for _ in 0..30:
        let ys = [rand(0, 9) for _ in 0..rand(1, 8)]
        let k = rand(-10, 10)
        let n = ys.len
        assert ys.rotated_left(k) == [ys[(i + k) % n] for i in 0..n]
        var z = ys.clone()
        z.rotate_right(k)
        assert z == ys.rotated_right(k)

/// \(O(1)\). The pair the other way round: `(a, b).swap` is `(b, a)`, and labels stay in place,
/// so an edge `(u: a, v: b).swap` is `(u: b, v: a)`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `(u: 1, v: 2).swap == (u: 2, v: 1)`
@get pub fn swap<A>(self: (A, A)) -> (A, A):
    let (a, b) = self
    (b, a)

/// What `split(sep:)` cuts at: an element equal to a value, or one where a predicate holds
/// (decision 456).
pub trait Sep<T>:
    fn is_sep(self, x: T) -> Bool

/// A predicate cuts where it holds.
impl Rev<[D]T>:
    /// \(O(n)\) calls of `f`.
    ///
    /// The n + 1 suffix folds of the source, keyed by its own cuts: `suf[c]` folds everything right
    /// of cut `c` (`from`, or `f`'s identity, at the last cut), so `suf[a.cut_before(i)]` folds
    /// `a[i..]`. The folds run right to left, `suf[c] == f(a[c.r], suf[c.next])`, keeping the
    /// elements' left-to-right order inside each fold, so `f` need not be commutative. `all` is the
    /// whole fold, `suffix(b)` reads a stored fold, and `prefix(b)` and `fold` subtract, so they
    /// need `f` to be `+`.
    ///
    /// cost: \(O(n)\) calls of `f`; \(O(n)\) space
    /// empty: `domain D = 0..0; let a: [D]Int = []; a.rev.scan(+).all == 0`
    /// order: keyed by `D.cuts`, right to left
    /// traps: never (beyond what `f` traps on); see `fold` for range reads
    /// example: `domain D = 0..3; let a = [D] d => d.ord + 2; a.rev.scan(*).at(D(1).l) == 12`
    /// example: `domain D = 0..3; let a = [D] d => d.ord + 2; a.rev.scan(+).all == 9`
    /// example: `domain D = 0..3; let a: [D]Int = [3, 1, 4]; a.rev.scan(+).suffix(D(1).l) == 5`
    /// example: `domain D = 0..3; let a: [D]Int = [3, 1, 4]; a.rev.scan(max).suffix(D(2).l) == 4`
    pub fn scan<M: FromOp<T>>(self, f: M, from: ?T = None) -> Prefix<D, T> where D: Ordered =
        Prefix<D, T>.of_rev_scan(self.src, f, from)

impl<T> Sep<T> for T -> Bool:
    fn is_sep(self, x: T) -> Bool = self(x)

/// A character cuts where an equal character stands.
impl Sep<Char> for Char:
    fn is_sep(self, x: Char) -> Bool = x == self

/// An integer cuts where an equal integer stands.
impl Sep<Int> for Int:
    fn is_sep(self, x: Int) -> Bool = x == self

/// Pieces between separators: elements equal to `sep`, or where the predicate `sep` holds.
/// Separators are excluded and empty pieces kept, so `k` separators give `k + 1` pieces. On a
/// `Str` each piece is a view that prints and compares as a string; `Str(p)` copies it.
///
/// cost: \(O(n)\)
/// empty: `"".split(',').count() == 1`
/// order: left to right; each piece keeps the keys of `self`
/// traps: never
/// example: `[p.len for p in [1, 0, 0, 2].split(x => x == 0)] == [1, 0, 1]`
/// example: `[p.len for p in [1, 0, 0, 2].split(0)] == [1, 0, 1]`
/// example: `[Str(p) for p in "a,b,,c".split(',')] == ["a", "b", "", "c"]`
/// example: `"".split(',').count() == 1`
pub fn split<S: Indexed<D, T>, D: Domain, T, P: Sep<T>>(self: S, sep: P) -> yields Slice<S>:
    var i = 0
    for j in 0..self.len:
        if sep.is_sep(self.nth(j)):
            yield piece(self, i, j)
            i = j + 1
    yield piece(self, i, self.len)

/// The ordinals `i..j` of `s` as a slice keeping its keys; an empty piece sits at key `i` when there is one.
fn clamp_count(k: Int, n: Int) -> Int:
    assert k >= 0, "a count must not be negative"
    if k < n: k else: n

fn piece<S: Indexed<D, T>, D: Domain, T>(s: S, i: Int, j: Int) -> Slice<S>:
    let n = s.len
    if i < j: return s[s.keys.nth(i)..=s.keys.nth(j - 1)]
    if i < n:
        let k = s.keys.nth(i)
        return s[k..k]
    if n == 0: return s[..]
    let k = s.keys.nth(n - 1)
    s[k..k]

/// Which end of a piece a two-pointer result is keyed by.
pub enum Anchor = Start | End
/// State a window can add to at the back and remove from at the front. `pop(x)` is passed the
/// element leaving the front. `Window(op)`, `Counter<K, Int>`, `Tracked` and `Undoing` implement it.
pub trait Track<T>:
    fn push(mut self, x: T)
    fn pop(mut self, x: T)

/// Two pointers: for each right end, the longest piece ending there that is `ok`, fed through
/// `state: Track`, as its positions.
///
/// Read the piece as `a[w]`, like [`spans`](spans). `ok` must be
/// monotone under shrinking. `anchor: Anchor.Start` keys by left
/// end instead: the longest piece starting at each position, as greedy jumps want. \(O(n)\) amortized.
/// Ties can't arise: each key has one longest piece.
///
/// cost: \(O(n)\) amortized pushes and pops on `state`
/// empty: `let e: List<Int> = []; e.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 1) == []`
/// ties: `[5, 5].maximal(state: Counter<Int, Int>(), ok: c => c.len <= 1).map(w => w.len) == [1, 2]`
/// order: keyed by `D`, by right end (or left end with `Anchor.Start`)
/// traps: never in release; debug traps when `ok` fails on a single element after a shrink
/// example: `[1, 1, 2].maximal(state: Counter<Int, Int>(), ok: c => c.len <= 1).map(w => w.len) == [1, 2, 1]`
pub fn maximal<D, T, St: Track<T>>(
    self: [D]T,
    state: St,
    ok: St -> Bool,
    anchor: Anchor = Anchor.End,
) -> [D]Range<D>:
    let n = self.len
    var st = state
    var out: [D]Range<D> = D.cuts.nth(0)..D.cuts.nth(0)
    match anchor:
        Anchor.End =>
            var l = 0
            for r in 0..n:
                st.push(self[D.nth(r)])
                while l <= r and not ok(st):
                    st.pop(self[D.nth(l)])
                    l += 1
                out[D.nth(r)] = D.cuts.nth(l)..D.cuts.nth(r + 1)
        Anchor.Start =>
            var l = 0
            for r in 0..n:
                st.push(self[D.nth(r)])
                while l <= r and not ok(st):
                    out[D.nth(l)] = D.cuts.nth(l)..D.cuts.nth(r)
                    st.pop(self[D.nth(l)])
                    l += 1
            while l < n:
                out[D.nth(l)] = D.cuts.nth(l)..D.cuts.nth(n)
                l += 1
    out

/// For each right end, the shortest piece ending there that is `ok` (its positions), or None,
/// keyed by that
/// end like the ring forms; `anchor: Anchor.Start` keys by left end. \(O(n)\) amortized.
/// Ties can't arise: each key has one shortest piece.
///
/// cost: \(O(n)\) amortized pushes and pops on `state`
/// empty: `let sum = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); let e: List<Int> = []; e.minimal(state: sum, ok: w => w.state >= 3) == []`
/// ties: `let sum = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); [3, 3].minimal(state: sum, ok: w => w.state >= 3).map(w => w.map(.len)) == [Some(1), Some(1)]`
/// order: keyed by `D`, by right end (or left end with `Anchor.Start`)
/// traps: never
///
/// ```cp
/// let sum = track(0, (mut s, x) => s += x, (mut s, x) => s -= x)
/// [1, 2, 3].minimal(state: sum, ok: w => w.state >= 3).map(w => w.map(.len))
///     == [None, Some(2), Some(1)]
/// ```
pub fn minimal<D, T, St: Track<T>>(
    self: [D]T,
    state: St,
    ok: St -> Bool,
    anchor: Anchor = Anchor.End,
) -> [D]?Range<D>:
    let n = self.len
    var st = state
    var out: [D]?Range<D> = None
    match anchor:
        Anchor.End =>
            var l = 0
            var best_l: ?Int = None
            for r in 0..n:
                st.push(self[D.nth(r)])
                while l <= r and ok(st):
                    best_l = l
                    st.pop(self[D.nth(l)])
                    l += 1
                if let bl = best_l: out[D.nth(r)] = Some(D.cuts.nth(bl)..D.cuts.nth(r + 1))
        Anchor.Start =>
            var l = 0
            for r in 0..n:
                st.push(self[D.nth(r)])
                while l <= r and ok(st):
                    out[D.nth(l)] = Some(D.cuts.nth(l)..D.cuts.nth(r + 1))
                    st.pop(self[D.nth(l)])
                    l += 1
    out

/// \(O(1)\) to make. `a` read as a cycle: windows and two pointers wrap around the end, and every
/// element is still a position of `a`. Replaces doubling the array.
///
/// cost: \(O(1)\)
/// empty: `let e: List<Int> = []; e.ring.src.len == 0`
/// traps: never
/// example: `[1, 2, 3].ring.next(2) == 0`
@get pub fn ring<D, T>(self: [D]T) -> Ring<D, T> = Ring(self)

/// `a.ring`; see `ring`.
pub struct Ring<D, T>(pub src: [D]T):
    /// The position `d` steps after `i` (before it when `d < 0`), wrapping: a forced-online
    /// decode `(v + last - 1) % n + 1` is `V.ring.step(v, last)`. \(O(1)\).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.step(0, -1) == 2`
    pub fn step(self, i: D, d: Int) -> D = D.nth(((i.ord + d) % D.count + D.count) % D.count)

    /// The position after `i`, wrapping.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.next(2) == 0`
    pub fn next(self, i: D) -> D = self.step(i, 1)

    /// The position before `i`, wrapping.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.prev(0) == 2`
    pub fn prev(self, i: D) -> D = self.step(i, -1)

    /// All n positions, starting at `i` and wrapping. \(O(n)\) to walk.
    /// An empty ring has no position, so no call is possible.
    ///
    /// cost: \(O(1)\) per step
    /// empty: `domain D = 0..0; let e: [D]Int = []; e.ring.from(D.first!).count() == 0` (traps R0301)
    /// order: `i`, then each next position, n in all
    /// traps: never
    /// example: `[1, 2, 3].ring.from(1).collect() == [1, 2, 0]`
    pub fn from(self, i: D) -> yields D:
        for k in 0..D.count: yield self.step(i, k)

    /// The `len` positions from `start` on, wrapping (`len <= n`).
    ///
    /// cost: \(O(1)\)
    /// empty: `[1, 2, 3].ring.span(2, 0).len == 0`
    /// traps: when `len` is not in `0..=n`
    /// example: `[1, 2, 3].ring.span(2, 2).end == 0`
    pub fn span(self, start: D, len: Int) -> RingRange<D>:
        assert 0 <= len <= D.count
        RingRange<D>(at: Some(start), n: len)

    /// The n windows of `k` elements, one starting at each position, wrapping. \(O(n)\) to walk.
    /// An empty ring traps for every `k`: no `k` is in `1..=n`.
    ///
    /// A lazy view: walking it, either way, is \(O(1)\) per window, and `.last` is the window
    /// starting at the last position.
    ///
    /// cost: \(O(1)\) to build, \(O(1)\) per window walked
    /// empty: `let e: List<Int> = []; e.ring.windows(1).count() == 0` (traps R0401)
    /// order: by start position, increasing
    /// traps: when `k` is not in `1..=n`
    /// example: `[1, 2, 3].ring.windows(2).count() == 3`
    pub fn windows(self, k: Int) -> RingWindows<D, T>:
        assert 1 <= k <= D.count
        RingWindows(self, k)

    /// Two pointers round the cycle: for each position as the last element, the longest
    /// wrapping piece (at most n elements) ending there that is `ok`, fed through
    /// `state: Track`. \(O(n)\) pushes and pops (two laps).
    /// Ties can't arise: each last element has one longest piece.
    ///
    /// cost: \(O(n)\) pushes and pops on `state`
    /// empty: `let e: List<Int> = []; e.ring.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 2) == []`
    /// order: keyed by `D`, by last element
    /// traps: never
    /// example: `[1, 1, 2].ring.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 2).map(w => w.len) == [3, 3, 3]`
    /// ties: `[5, 5].ring.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 1).map(w => w.len) == [2, 2]`
    pub fn maximal<St: Track<T>>(self, state: St, ok: St -> Bool) -> [D]RingSlice<D, T>:
        let n = D.count
        var st = state
        var out: List<RingSlice<D, T>> = []
        var l = 0
        for t in 0..2 * n:
            st.push(src[D.nth(t % n)])
            while l <= t and (t - l + 1 > n or not ok(st)):
                st.pop(src[D.nth(l % n)])
                l += 1
            if t >= n: out.push(RingSlice<D, T>(rr: self.span(D.nth(l % n), t - l + 1)))
        [D] i => out[i.ord]

    /// For each position as the last element, the shortest wrapping piece ending there that is
    /// `ok`, or None. \(O(n)\) pushes and pops (two laps).
    /// Ties can't arise: each last element has one shortest piece.
    ///
    /// cost: \(O(n)\) pushes and pops on `state`
    /// empty: `let e: List<Int> = []; e.ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 3) == []`
    /// order: keyed by `D`, by last element
    /// traps: never
    /// example: `[2, 0, 1].ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 3).map(w => w.map(.len)) == [Some(2), Some(3), Some(3)]`
    /// ties: `[3, 3].ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= 3).map(w => w.map(.len)) == [Some(1), Some(1)]`
    pub fn minimal<St: Track<T>>(self, state: St, ok: St -> Bool) -> [D]?RingSlice<D, T>:
        let n = D.count
        var st = state
        var out: [D]?RingSlice<D, T> = [D] _ => None
        var l = 0
        var best: ?Int = None
        for t in 0..2 * n:
            st.push(src[D.nth(t % n)])
            while l <= t and t - l + 1 > n:
                st.pop(src[D.nth(l % n)])
                l += 1
            while l <= t and ok(st):
                best = Some(l)
                st.pop(src[D.nth(l % n)])
                l += 1
            if let b = best, n <= t < b + n:
                out[D.nth(t - n)] = Some(RingSlice<D, T>(rr: self.span(D.nth(b % n), t - b + 1)))
        out

/// A contiguous piece of a ring, possibly wrapping past the end, at most n elements.
/// The windows of `k` elements round a `Ring`, one per start position: `r.windows(k)`.
pub struct RingWindows<D, T>(
    pub src: Ring<D, T>,
    k: Int,
)

impl Seq<RingSlice<D, T>> for RingWindows<D, T>:
    fn iter(self) -> yields RingSlice<D, T>:
        for i in D: yield RingSlice<D, T>(rr: src.span(i, k))

impl Bidir<RingSlice<D, T>> for RingWindows<D, T>:
    fn iter_rev(self) -> yields RingSlice<D, T>:
        for j in (0..D.count).rev: yield RingSlice<D, T>(rr: src.span(D.nth(j), k))

impl Sized for RingWindows<D, T>:
    @get fn len(self) -> Int = D.count

impl<D, T> Seq<D> for RingSlice<D, T>:
    /// \(O(\mathit{len})\). Visit the window's positions in the source; read its elements as
    /// `a[k]`. Same walk as `keys`.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `[1, 2, 3].ring.windows(0).all(w => w.each().collect() == [])`
    /// order: ring order from `start`, wrapping past the last position
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).last!.each().collect() == [2, 0]`
    fn iter(self) -> yields D:
        for k in rr.keys: yield k

pub struct RingSlice<D, T>():
    rr: RingRange<D> = RingRange<D>()

    /// \(O(1)\). Its positions, without the array.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.range.len == 2`
    @get pub fn range(self) -> RingRange<D> = rr

    /// The number of elements in the window.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[w.len for w in [1, 2, 3].ring.windows(2)] == [2, 2, 2]`
    @get pub fn len(self) -> Int = rr.len

    /// Whether the window covers no position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `not [1, 2, 3].ring.windows(2).first!.is_empty`
    @get pub fn is_empty(self) -> Bool = rr.len == 0

    /// \(O(1)\). The first position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.start == 0`
    @get pub fn start(self) -> D = rr.start

    /// \(O(1)\). The last position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.end == 1`
    @get pub fn end(self) -> D = rr.end

    /// \(O(1)\) to make. Its positions in ring order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) per step
    /// order: ring order from `start`
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.keys.collect() == [0, 1]`
    @get pub fn keys(self) -> yields D:
        for k in rr.keys: yield k

/// The non-empty ranges whose every element is strictly below both elements just outside it
/// (both must exist).
///
/// At most 2n of them, left ends ascending, from one monotone-stack pass;
/// the "walls" shape of grid problems such as IOI 2019 Rectangles. \(O(n)\) to walk.
///
/// cost: \(O(n)\) to walk
/// empty: `let e: List<Int> = []; e.valleys().count() == 0`
/// order: left ends ascending, then shorter first
/// traps: never
/// example: `[3, 1, 2, 5].valleys().collect() == [1..2, 1..3]`
pub fn valleys<D, T: Ord>(self: [D]T) -> yields Range<D>:
    let n = D.count
    var ends: List<List<Int>> = [[] for _ in 0..n]
    var st: List<Int> = []
    for j in 0..n:
        let x = self[D.nth(j)]
        while st.len > 0 and self[D.nth(st.last!)] < x:
            st.pop()
            if st.len > 0: ends[st.last!].push(j)
        if st.len > 0 and self[D.nth(st.last!)] == x: st.pop()
        st.push(j)
    for l in 0..n:
        for r in ends[l]: yield D.nth(l + 1)..D.nth(r)

/// For each position, the widest piece where it is the minimum of `by` (ties: the left one
/// extends). \(O(n)\).
///
/// cost: \(O(n)\)
/// empty: `let e: List<Int> = []; e.spans() == []`
/// ties: `[1, 1].spans() == [0..2, 1..2]`
/// order: keyed by `D`
/// traps: never
/// example: `[2, 1, 1].spans() == [0..1, 0..3, 2..3]`
pub fn spans<D, T, K>(self: [D]T, by: T -> K = x => x) -> [D]Range<D>:
    let left = self.prev_smaller(by:, strict: false)
    let right = self.next_smaller(by:, strict: true)
    [D] i => (if let l = left[i]: l.r else: D.cuts.nth(0))..(
        if let r = right[i]: r.l else: D.cuts.nth(D.count)
    )

/// Nearest earlier position with a smaller key (`strict: false`: smaller or equal).
///
/// cost: \(O(n)\)
/// empty: `let e: List<Int> = []; e.prev_smaller() == []`
/// ties: `[1, 1].prev_smaller() == [None, None] and [1, 1].prev_smaller(strict: false) == [None, Some(0)]`
/// order: keyed by `D`
/// traps: never
/// example: `[2, 1, 3].prev_smaller() == [None, None, Some(1)]`
pub fn prev_smaller<D, T, K: Ord>(self: [D]T, by: T -> K = x => x, strict: Bool = true) -> [D]?D:
    var out: [D]?D = None
    var st: List<D> = []
    for i in D:
        while st.len > 0 and (
            if strict: by(self[st.last!]) >= by(self[i]) else: by(self[st.last!]) > by(self[i])
        ):
            let _ = st.pop()
        if st.len > 0: out[i] = st.last!
        st.push(i)
    out

/// Nearest later position with a smaller key.
///
/// cost: \(O(n)\)
/// empty: `let e: List<Int> = []; e.next_smaller() == []`
/// ties: `[1, 1].next_smaller() == [None, None] and [1, 1].next_smaller(strict: false) == [Some(1), None]`
/// order: keyed by `D`
/// traps: never
/// example: `[2, 1, 3].next_smaller() == [Some(1), None, None]`
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] < self[i]:\n    ans[st.pop()!] = i", to: "self.next_smaller(by: x => -x)", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] < self[i]:\n    let j = st.last!\n    st.pop()\n    ans[j] = i", to: "self.next_smaller(by: x => -x)", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] <= self[i]:\n    ans[st.pop()!] = i", to: "self.next_smaller(by: x => -x, strict: false)", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] <= self[i]:\n    let j = st.last!\n    st.pop()\n    ans[j] = i", to: "self.next_smaller(by: x => -x, strict: false)", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] > self[i]:\n    ans[st.pop()!] = i", to: "self.next_smaller()", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] > self[i]:\n    let j = st.last!\n    st.pop()\n    ans[j] = i", to: "self.next_smaller()", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] >= self[i]:\n    ans[st.pop()!] = i", to: "self.next_smaller(strict: false)", holes: "st, i, ans")
@lint(monotone_stack_by_hand, from: "while st.len > 0 and self[st.last!] >= self[i]:\n    let j = st.last!\n    st.pop()\n    ans[j] = i", to: "self.next_smaller(strict: false)", holes: "st, i, ans")
pub fn next_smaller<D, T, K: Ord>(self: [D]T, by: T -> K = x => x, strict: Bool = true) -> [D]?D:
    var out: [D]?D = None
    var st: List<D> = []
    for k in (0..D.count).rev:
        let i = D.nth(k)
        while st.len > 0 and (
            if strict: by(self[st.last!]) >= by(self[i]) else: by(self[st.last!]) > by(self[i])
        ):
            let _ = st.pop()
        if st.len > 0: out[i] = st.last!
        st.push(i)
    out

/// Nearest earlier position with an equal key.
///
/// cost: \(O(n)\) expected
/// empty: `let e: List<Int> = []; e.prev_same() == []`
/// order: keyed by `D`
/// traps: never
/// example: `[1, 2, 1].prev_same() == [None, None, Some(0)]`
pub fn prev_same<D, T, K: Hash + Eq>(self: [D]T, by: T -> K = x => x) -> [D]?D:
    var out: [D]?D = None
    var seen = HashMap<K, D>()
    for i in D:
        let k = by(self[i])
        out[i] = seen.get(k)
        seen[k] = i
    out

/// Nearest later position with an equal key.
///
/// cost: \(O(n)\) expected
/// empty: `let e: List<Int> = []; e.next_same() == []`
/// order: keyed by `D`
/// traps: never
/// example: `[1, 2, 1].next_same() == [Some(2), None, None]`
pub fn next_same<D, T, K: Hash + Eq>(self: [D]T, by: T -> K = x => x) -> [D]?D:
    var out: [D]?D = None
    var seen = HashMap<K, D>()
    for j in (0..D.count).rev:
        let i = D.nth(j)
        let k = by(self[i])
        out[i] = seen.get(k)
        seen[k] = i
    out

/// The ranges no other range contains, sorted by left end (ties: the first kept). Equal ranges
/// keep one. The usual first step of interval DP over covers. \(O(n \log n)\).
///
/// cost: \(O(n \log n)\)
/// empty: `let e: List<Range<Int>> = []; e.outermost() == []`
/// ties: `[1..3, 0..1, 1..3].outermost() == [0..1, 1..3]`
/// order: by left end; a new `List`, keyed from 0
/// traps: never
/// example: `[0..5, 1..3, 4..8].outermost() == [0..5, 4..8]`
pub fn outermost<D, S: Seq<Range<D>>>(self: S) -> List<Range<D>>:
    var rs: List<Range<D>> = self.collect()
    rs.sort(by: r => r.start)
    // kept in place: a range stays when it reaches past the last kept one; one with the
    // same left end replaces that one when it is longer
    var w = 0
    for i in 0..rs.len:
        let r = rs[i]
        if w > 0 and rs[w - 1].start == r.start:
            if rs[w - 1].r < r.r: rs[w - 1] = r
        elif w == 0 or rs[w - 1].r < r.r:
            rs[w] = r
            w += 1
    while rs.len > w: rs.pop()
    rs

/// \(O(1)\). A list's keys: `0..xs.len`, in increasing order.
///
/// cost: \(O(1)\)
/// empty: `let e: List<Int> = []; e.keys == 0..0`
/// order: increasing
/// traps: never
/// example: `[5, 6, 7].keys == 0..3`
@get pub fn keys<T>(self: List<T>) -> Range<Int> = 0..self.len

@test
fn dedup_first_some_and_group():
    var a = [1, 1, 2, 1, 1]
    a.dedup()
    assert a == [1, 2, 1]
    var e: List<Int> = []
    e.dedup()
    assert e == []
    assert [1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None) == Some(40)
    assert [1, 2].first_some(x => if x > 3: Some(x) else: None) == None
    let g = [(1, 'a'), (2, 'b'), (1, 'c')].group(by: p => p.0, of: p => p.1)
    assert g[1] == ['a', 'c'] and g[2] == ['b']

@test
fn ring_steps_wrap_both_ways():
    domain V = 0..3
    let a: [V]Int = [1, 2, 3]
    let r = a.ring
    assert r.next(V(2)) == V(0) and r.prev(V(0)) == V(2)
    assert r.step(V(1), 7) == V(2) and r.step(V(2), -8) == V(0)
    assert r.from(V(1)).collect() == [V(1), V(2), V(0)]

/// `gen<Perm<V>>()`: every permutation of `V` equally likely.
impl<V> Random for Perm<V>:
    fn gen() -> Perm<V>:
        var p: [V]V = [V] v => v
        shuffle(mut p)
        p

@test
fn views_and_trackers_walk_and_show():
    let a = [5, 6, 7, 8]
    assert a[1..3].items.collect() == [(1, 6), (2, 7)]
    assert "{a[1..3]}" == "6 7"
    let st = a.step(2)
    assert st.each().collect() == [5, 7] and not st.is_empty
    assert "{st}" == "5 7"
    assert a[4..].step(2).is_empty
    let (domain C, xs) = compress([30, 10, 30])
    assert [v for (_, v) in xs.items] == [10, 30] and "{xs}" == "10 30"
    var t = OnlineLis<Int>()
    for x in [3, 1, 2]: t.push(x)
    assert "{t}" == "1 2"
    var s = MonoStack<Int, Int>()
    s.push(0, 5)
    s.push(1, 3)
    s.push(2, 4)
    assert s.items.collect() == [(0, 5), (2, 4)] and "{s}" == "0=5, 2=4"
    let w = [1, 2, 3].ring.windows(2).last!
    assert w.each().collect() == [2, 0] and "{w}" == "2 0" and not w.is_empty
    let r = [1, 2, 3].ring.span(2, 2)
    assert r.each().collect() == [2, 0] and "{r}" == "2 0" and not r.is_empty

/// A view printed as a list is: its elements joined by `sep` when given, else by each one's gap
/// (R10.3.1). It writes them without `print`'s `Fmt` (precision, base, `none:`): the frozen old
/// `cpl` reads this file too, and importing std.show breaks it; see migration.md.
fn write_seq<S: Seq<T>, T: Show>(s: S, mut out: Str, sep: ?Str):
    var first = true
    for x in s:
        if not first:
            if let g = sep: out.append(g)
            else: x.gap(mut out)
        x.write_to(mut out, None)
        first = false

impl<S: Bidir<T>, T: Show> Show for Rev<S>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl Show for Filtered<S: Seq<T>, T: Show>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl Show for Mapped<S: Seq<T>, T, U: Show>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<S: Indexed<K, T>, K: Domain, T: Show> Show for Rebased<S>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<S: Indexed<K, T>, K: Domain, T: Show> Show for Slice<S>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl Show for Row<R, C, T: Show>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl Show for Col<R, C, T: Show>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl Show for Rect<R, C, T: Show>:
    fn write_to(self, mut out: Str, sep: ?Str) = write_seq(self, mut out, sep)
    fn gap(self, mut out: Str):
        out.push('\n')

impl Seq<T> for Stepped<S, T>:
    fn iter(self) -> yields T:
        for j in 0..self.len: yield self.nth(j)

impl Bidir<T> for Stepped<S, T>:
    fn iter_rev(self) -> yields T:
        var j = self.len
        while j > 0:
            j -= 1
            yield self.nth(j)

impl Show for Stepped<S, T: Show>:
    fn show(v):
        var out = ""
        var first = true
        for x in v.each():
            if not first: out += " "
            out += "{x}"
            first = false
        out

impl Show for Compressed<D, T: Show>:
    fn show(v):
        var out = ""
        for (_, x) in v.items:
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for OnlineLis<T: Show>:
    fn show(v):
        var out = ""
        for (_, x) in v.items:
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for MonoStack<D: Show, T: Show>:
    fn show(v):
        var out = ""
        for (i, x) in v.items:
            if not out.is_empty: out += ", "
            out += "{i}={x}"
        out

impl Show for RingSlice<D: Show, T>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for StepKeys:
    /// The positions the walk visits, in order, space-separated.
    ///
    /// example: `"{[3, 1, 2, 4, 5].step(2).keys}" == "0 2 4"`
    fn show(s):
        var out = ""
        for j in 0..s.len:
            if not out.is_empty: out += " "
            out += "{s.first + s.from + j * s.k}"
        out

// The hash tables' sequence impls live with the traits: `std.hash` sits below `std.seq`.
impl<K: Hash + Eq, V> IndexGet<K, V> for HashMap<K, V>:
    fn at(self, k: K) -> V:
        check_key(k in self)
        self.get(k)!

impl<K: Hash + Eq, V> Seq<(K, V)> for HashMap<K, V>:
    fn iter(self) -> yields (K, V):
        for p in self.items: yield p

impl<K: Hash + Eq, V> Sized for HashMap<K, V>:
    @get fn len(self) -> Int = self.len

impl<T: Hash + Eq> Seq<T> for HashSet<T>:
    fn iter(self) -> yields T:
        for x in self.items: yield x

impl<T: Hash + Eq> Sized for HashSet<T>:
    @get fn len(self) -> Int = self.len
