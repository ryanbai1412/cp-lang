//! Sequences: reductions, scans, runs, searching, permutations, monotone stacks, and compression.
//!
//! - Reductions & extrema: `prod`, `is_sorted`, `min`/`max` (with `by:`), `argmax`/`argmin` (with `by:`).
//! - Binary & unimodal search: moved to `std.search` (`bsearch_real`, `ternary_max`/`ternary_min`, `ternary_max_real`/`ternary_min_real`, `parallel_bsearch`).
//! - Sorted lookup: `first_ge`/`first_gt`/`last_le`/`last_lt` (with optional `by:`), returning `?D` on `[D]T`, `?Int` on `List<T>`, and `?(K, V)` on `Map<K, V>`.
//! - Runs, chunking & windows: `runs`, `run_ranges`, `chunk_by`, `buckets`, `windows`, `chunks`, `pairs`, `with_prev`, `cyclic_pairs`, `zip`.
//! - Scans & folds: `scan`, `suffix_scan`, `excl_fold` (cut-indexed `prefix_fold`/`suffix_fold` live in `std.prefix`).
//! - Reordering & selection: `reverse`, `rotate_left`/`rotate_right`, `dedup`/`deduped`, `partition`, `select`, `merge_sorted`.
//! - Permutations: `next_permutation`, `prev_permutation`, `inverse`, `compose`, `perm_cycles`, `perm_pow`, `inversions`.
//! - Monotone stack & sliding extrema: `prev_less`, `next_less`, `prev_greater`, `next_greater`, `sliding_min`, `sliding_max`.
//! - Coordinate compression: `let (domain C, xs) = compress(all)` -> `Compressed<C, T>` (`at`, `try_at`, `contains`, `first_ge`, `first_gt`, `last_le`, `last_lt`).


/// Every value of `V` once, in some order. `cpl stress` makes and shrinks it as a permutation.
pub type Perm<V> = [V]V

/// Product of all elements of `self` (`T.one` when empty). Works on any `Seq` (`Str`, `List`, `[D]T`, `Range`, views) of `Int`, `Mod<P>`, `F64`.
///
/// cost: O(n)
/// empty: `T.one`
/// traps: never (overflow wraps in release, traps in debug)
/// example: `[2, 3, 4].prod() == 24`
pub fn prod<S: Seq, T>(self: S) -> T:
    var out = T.one
    for x in self: out *= x
    out

/// Sum of all elements (`T.zero` when empty), on any `Seq` of a type with `+`; `of:` maps each
/// element first (`xs.sum(of: x => x * x)`). O(n).
///
/// cost: O(n)
/// empty: `U.zero`
/// traps: never (overflow wraps in release, traps in debug)
/// example: `[1, 2, 3].sum(of: x => x * x) == 14`
pub fn sum<S: Seq, T, U>(self: S, of: T -> U = x => x) -> U:
    var out = U.zero
    for x in self: out += of(x)
    out

/// The average of `of` over the elements as `F64`; `None` when empty.
///
/// cost: O(n)
/// empty: `None`
/// traps: never
/// example: `[1, 2].mean() == Some(1.5)`
pub fn mean<S: Seq, T>(self: S, of: T -> F64 = x => F64(x)) -> ?F64:
    var total = 0.0
    var n = 0
    for x in self:
        total += of(x)
        n += 1
    if n == 0: return None
    Some(total / F64(n))

/// The least element by `by`, the first on ties; `None` when empty. A method call (`s.min()`)
/// picks this fold; a plain call with two values (`min(a, b)`) is the core binary `min`. `a.keys.min(by: i => a[i])`
/// is argmin. O(n).
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in sequence order
/// traps: never
/// example: `[3, 1, 1].min() == Some(1)`
/// example: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`
pub fn min<S: Seq, T, K>(self: S, by: T -> K = x => x) -> ?T:
    var best: ?T = None
    for x in self:
        if let b = best:
            if by(x) < by(b): best = Some(x)
        else: best = Some(x)
    best

/// The greatest element by `by`, the first on ties; `None` when empty.
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in sequence order
/// traps: never
/// example: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`
pub fn max<S: Seq, T, K>(self: S, by: T -> K = x => x) -> ?T:
    var best: ?T = None
    for x in self:
        if let b = best:
            if by(x) > by(b): best = Some(x)
        else: best = Some(x)
    best

/// How many elements satisfy `p`.
///
/// cost: O(n)
/// empty: `0`
/// traps: never
/// example: `[1, 2, 3, 4].count(x => x % 2 == 0) == 2`
pub fn count<S: Seq, T>(self: S, p: T -> Bool) -> Int:
    var n = 0
    for x in self:
        if p(x): n += 1
    n

/// Whether some element satisfies `p`; stops at the first.
///
/// cost: O(n), stopping at the first match
/// empty: `false`
/// traps: never
/// example: `[1, 2].any(x => x > 1) == true`
pub fn any<S: Seq, T>(self: S, p: T -> Bool) -> Bool:
    for x in self:
        if p(x): return true
    false

/// Whether every element satisfies `p`; stops at the first that doesn't.
///
/// cost: O(n), stopping at the first failure
/// empty: `true`
/// traps: never
/// example: `[1, 2].all(x => x > 1) == false`
pub fn all<S: Seq, T>(self: S, p: T -> Bool) -> Bool:
    for x in self:
        if not p(x): return false
    true

/// Whether `self` is non-decreasing (optionally under key `by:`). Works on any `Seq`.
///
/// cost: O(n)
/// empty: `true`; a single element is sorted too
/// traps: never
/// example: `[1, 1, 2].is_sorted() == true`
pub fn is_sorted<S: Seq, T, K>(self: S, by: T -> K = x => x) -> Bool:
    var prev: ?K = None
    for x in self:
        let k = by(x)
        if let p = prev, k < p: return false
        prev = k
    true

// --- Adapters, ends, and sorting (R07.2.8, R07.4) ---

/// O(1) to make. A view of the elements satisfying `p`, in order. Keys survive: each element
/// keeps its source key, so `a.filter(p).items` gives `(i, a[i])` with `a`'s own `i`. `p` runs
/// as the view is walked, once per element per walk. Bidirectional when `self` is.
///
/// cost: O(1) to make; O(n) calls of `p` per walk
/// empty: an empty view
/// order: source order
/// traps: never (beyond what `p` traps on)
/// example: `[1, 2, 3, 4].filter(x => x % 2 == 0).collect() == [2, 4]`
/// example: `[5, 6, 7].filter(x => x > 5).items.collect() == [(1, 6), (2, 7)]`
pub fn filter<S: Seq, T>(self: S, p: T -> Bool) -> Filtered<S> = todo

/// O(1) to make. A view giving `f(x)` for each element. Indexed when `self` is, with the same
/// keys: `a.map(f)[i]` is `f(a[i])`, calling `f` on every read (`collect()` once to cache).
///
/// cost: O(1) to make; one call of `f` per element read
/// empty: an empty view
/// order: source order, source keys
/// traps: never (beyond what `f` traps on)
/// example: `[1, 2, 3].map(x => x * x).collect() == [1, 4, 9]`
/// example: `domain V = 1..=3; let a: [V]Int = [4, 5, 6]; a.map(x => -x)[V(2)] == -5`
pub fn map<S: Seq, T, U>(self: S, f: T -> U) -> Mapped<S, U> = todo

/// O(1). A view walking `self` backwards. Keys survive: `a.rev[i]` is still `a[i]`; only the
/// walk order changes.
///
/// cost: O(1)
/// empty: an empty view
/// order: reverse source order, source keys
/// traps: never
/// example: `[1, 2, 3].rev.collect() == [3, 2, 1]`
@get pub fn rev<S: Bidir>(self: S) -> Rev<S> = todo

/// O(1). A view of `(key, element)` pairs with the source's own keys.
///
/// cost: O(1)
/// empty: an empty view
/// order: source order
/// traps: never
/// example: `[5, 6][1..].items.collect() == [(1, 6)]`
@get pub fn items<S: Keyed, K, T>(self: S) -> Items<S> = todo

/// O(1). The first element, `None` when empty. For the first element satisfying a predicate,
/// use `find(p)`; for its key, `keys.find(p)`.
///
/// cost: O(1)
/// empty: `None`
/// ties: not applicable; position decides
/// traps: never
/// example: `[4, 5].first == Some(4)`
@get pub fn first<S: Seq, T>(self: S) -> ?T = todo

/// O(1). The last element, `None` when empty. For the last element satisfying a predicate,
/// use `rfind(p)`.
///
/// cost: O(1)
/// empty: `None`
/// ties: not applicable; position decides
/// traps: never
/// example: `[4, 5].last == Some(5)`
@get pub fn last<S: Bidir, T>(self: S) -> ?T = todo

/// The last element satisfying `p`, `None` if None does; walks backwards and stops there.
///
/// cost: O(n) calls of `p` at most
/// empty: `None`
/// ties: the last in sequence order
/// traps: never (beyond what `p` traps on)
/// example: `[1, 4, 2, 5].rfind(x => x % 2 == 0) == Some(2)`
pub fn rfind<S: Bidir, T>(self: S, p: T -> Bool) -> ?T:
    for x in self.rev:
        if p(x): return Some(x)
    None

/// Sort `self` in place by `by` (descending when `desc`). Stable: equal keys keep their order.
/// Works on any `IndexedMut`, including slices: `a[l..=r].sort()`.
///
/// cost: O(n log n) comparisons, O(n) extra space
/// empty: does nothing
/// ties: stable; equal keys keep their relative order
/// traps: never (beyond what `by` traps on)
/// example: `var a = [3, 1, 2]; a.sort(); a == [1, 2, 3]`
/// example: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: .0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`
pub fn sort<S: IndexedMut, T, K: Ord>(mut self: S, by: T -> K = x => x, desc: Bool = false) = todo

/// The elements of `self` sorted by `by`: the value form of `sort`. A new `List`, keyed from 0;
/// to keep keys, sort the keys instead (`a.keys.sorted(by: i => a[i])`).
///
/// cost: O(n log n) comparisons, O(n) space
/// empty: `[]`
/// ties: stable; equal keys keep their relative order
/// order: increasing `by` (decreasing when `desc`), keyed from 0
/// traps: never (beyond what `by` traps on)
/// example: `[3, 1, 2].sorted() == [1, 2, 3]`
/// example: `[3, 1, 2].sorted(desc: true) == [3, 2, 1]`
pub fn sorted<S: Seq, T, K: Ord>(self: S, by: T -> K = x => x, desc: Bool = false) -> List<T> = todo

/// O(n). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).
/// Works on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).
///
/// cost: O(n), O(1) extra space
/// empty: does nothing
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

/// O(n). Drop each value equal to the one before it, in place: a sorted list keeps one of each.
///
/// cost: O(n), O(1) extra space
/// empty: does nothing
/// ties: keeps the first of each run of equal values
/// traps: never
/// example: `var a = [1, 1, 2, 1]; a.dedup(); a == [1, 2, 1]`
pub fn dedup<T>(mut self: List<T>):
    var n = 0
    for i in 0..self.len:
        if n == 0 or self[i] != self[n - 1]:
            self[n] = self[i]
            n += 1
    while self.len > n: let _ = self.pop()

/// `self` without each value equal to the one before it: the value form of `dedup`, as `sorted` is of `sort`.
/// Works on any `Seq`. O(n) time without hashing.
///
/// cost: O(n)
/// empty: `[]`
/// order: first of each run, in input order; a new `List`, keyed from 0
/// traps: never
/// example: `[1, 1, 2, 1].deduped() == [1, 2, 1]`
pub fn deduped<S: Seq, T>(self: S) -> List<T>:
    var out: List<T> = []
    for x in self:
        if out.is_empty or out.last! != x:
            out.push(x)
    out

/// O(n). The positions holding a new strict maximum, left to right: the first position, then
/// each one greater than everything before it. `record_highs(by: f)` compares `f(x)`.
///
/// cost: O(1) per step
/// empty: yields nothing
/// ties: an equal value is not a new high
/// order: increasing keys of `self`
/// traps: never
/// example: `[3, 1, 4, 4, 5].record_highs().collect() == [0, 2, 4]`
pub fn record_highs<D, T, K>(self: [D]T, by: T -> K = x => x) -> yields D:
    var best: ?K = None
    for i in D:
        let k = by(self[i])
        if (if let b = best: k > b else: true):
            best = k
            yield i

/// Distinct values of `self` in first-seen order (optionally keyed by `by:`).
/// Works on any `Seq`. O(n) expected time, O(n) space via `HashSet`. Unlike
/// `dedup` / `deduped` (adjacent duplicates only, O(n) time without hashing),
/// `distinct` drops duplicates anywhere in the sequence.
///
/// cost: O(n) expected, O(n) extra space
/// empty: `[]`
/// ties: keeps the first occurrence of each key
/// order: first-seen order; a new `List`, keyed from 0
/// traps: never
/// example: `[3, 1, 3, 2, 1].distinct() == [3, 1, 2]`
pub fn distinct<S: Seq, T, K>(self: S, by: T -> K = x => x) -> List<T>:
    var seen = HashSet<K>()
    var out: List<T> = []
    for x in self:
        if seen.add(by(x)):
            out.push(x)
    out

/// The values of `self` that `keep` holds for, then the ones it does not, each in the order they came.
///
/// cost: O(n)
/// empty: `([], [])`
/// order: each list keeps input order
/// traps: never
/// example: `[1, 2, 3, 4].partition(x => x % 2 == 0) == ([2, 4], [1, 3])`
pub fn partition<S: Seq, T>(self: S, keep: T -> Bool) -> (List<T>, List<T>):
    var (yes, no): (List<T>, List<T>) = ([], [])
    for x in self:
        if keep(x): yes.push(x)
        else: no.push(x)
    (yes, no)

/// O(n) expected, O(n log n) worst case, O(1) extra space. Introselect partitioning `self` in place
/// so `self.nth(k)` holds the element that would sit at `k` in sorted order (by `by:` or `cmp:`).
///
/// cost: O(n) expected, O(n log n) worst case, O(1) extra space
/// empty: traps (no `k` is valid)
/// ties: equal elements may land in any order on either side of `k`
/// traps: when `k` is not in `0..self.len`
/// example: `var a = [5, 1, 4, 2]; a.select(1); a.nth(1) == 2`
pub fn select<S: IndexedMut, T, K>(
    mut self: S,
    k: Int,
    by: T -> K = x => x,
    cmp: ?((T, T) -> Bool) = None,
):
    let n = self.len
    let _ = self.at(k)
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
        if less(self.at(mid), self.at(lo)): self.swap_idx(mid, lo)
        if less(self.at(hi - 1), self.at(lo)): self.swap_idx(hi - 1, lo)
        if less(self.at(hi - 1), self.at(mid)): self.swap_idx(hi - 1, mid)
        let pivot = self.at(mid)
        var (i, j) = (lo, hi - 1)
        while i <= j:
            while less(self.at(i), pivot): i += 1
            while less(pivot, self.at(j)): j -= 1
            if i <= j:
                self.swap_idx(i, j)
                i += 1
                j -= 1
        if k <= j: hi = j + 1
        elif k >= i: lo = i
        else: return

fn heap_select<S: IndexedMut, T>(mut self: S, lo: Int, hi: Int, k: Int, less: (T, T) -> Bool):
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

fn sift_down<S: IndexedMut, T>(mut self: S, lo: Int, from: Int, m: Int, less: (T, T) -> Bool):
    var i = from
    while true:
        var c = 2 * i + 1
        if c >= m: return
        if c + 1 < m and less(self.at(lo + c), self.at(lo + c + 1)): c += 1
        if not less(self.at(lo + i), self.at(lo + c)): return
        self.swap_idx(lo + i, lo + c)
        i = c

/// O(|a| + |b|), O(|a| + |b|) memory. Stable merge of two sorted sequences (keyed by `by:`),
/// taking from `a` before `b` on ties. Works on any `Indexed` sequences.
///
/// cost: O(|a| + |b|), O(|a| + |b|) memory
/// empty: the other input, copied
/// ties: `a` before `b`; within each input, input order (stable)
/// order: non-decreasing by `by`; a new `List`, keyed from 0
/// traps: never; unsorted inputs give an unspecified interleaving
/// example: `merge_sorted([1, 3], [2, 3]) == [1, 2, 3, 3]`
pub fn merge_sorted<A: Indexed, B: Indexed, T, K>(
    a: A,
    b: B,
    by: T -> K = x => x,
) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < a.len and j < b.len:
        if b.at(j) < a.at(i):
            out.push(b.at(j))
            j += 1
        else:
            out.push(a.at(i))
            i += 1
    while i < a.len:
        out.push(a.at(i))
        i += 1
    while j < b.len:
        out.push(b.at(j))
        j += 1
    out

/// O(1) per window. Borrowed length-`k` sliding slice views, left to right. Nothing when `k > self.len`.
///
/// cost: O(1) per window
/// empty: yields nothing; so does `k > self.len`
/// order: left to right; each window keeps the keys of `self`
/// traps: when `k < 1`
/// example: `[1, 2, 3].windows(2).map(w => w.sum()).collect() == [3, 5]`
pub fn windows<S: Indexed>(self: S, k: Int) -> yields Slice<S>:
    assert k >= 1, "a window holds at least one element"
    for i in 0..self.len - k + 1:
        var w: List<T> = []
        for j in i..i + k: w.push(self.at(j))
        yield w

/// O(1) per chunk. Borrowed non-overlapping slice views of length `k` (the last may be shorter).
///
/// cost: O(1) per chunk
/// empty: yields nothing
/// order: left to right; each chunk keeps the keys of `self`, the last may be shorter
/// traps: when `k < 1`
/// example: `[1, 2, 3].chunks(2).map(c => c.len).collect() == [2, 1]`
pub fn chunks<S: Indexed>(self: S, k: Int) -> yields Slice<S> = todo

/// Adjacent pairs `(prev, x)` of `self`, left to right: `[10, 20, 30].adjacent` gives
/// `(10, 20)` then `(20, 30)`. Works on any `Seq` (`List<T>`, `[D]T`, `Str`, `Range`, views). O(1) per step.
///
/// cost: O(1) per step
/// empty: yields nothing; so does a single element
/// order: left to right
/// traps: never
/// example: `[10, 20, 30].adjacent.collect() == [(10, 20), (20, 30)]`
@get pub fn adjacent<S: Seq, T>(self: S) -> yields (T, T) = todo

/// Every pair `(x, y)` with `x` before `y`: `[10, 20, 30].pairs()` gives `(10, 20)`, `(10, 30)`,
/// `(20, 30)`. O(n²) to walk; for adjacent pairs use `adjacent`.
///
/// cost: O(n²) to walk, O(1) per pair
/// empty: yields nothing
/// order: by the first element's position, then the second's
/// traps: never
/// example: `[1, 2, 3].pairs().count() == 3`
/// see: `adjacent` for neighbours, `Pair<V>` for a table keyed by unordered pairs
pub fn pairs<S: Seq, T>(self: S) -> yields (T, T):
    var prev: ?T = None
    for x in self:
        if let p = prev: yield (p, x)
        prev = x

/// Every element paired with the one before it, starting with `(None, first)`:
/// `[10, 20, 30].with_prev` gives `(None, 10)`, `(10, 20)`, `(20, 30)`. O(1) per step.
///
/// cost: O(1) per step
/// empty: yields nothing
/// order: left to right, starting with `(None, first)`
/// traps: never
/// example: `[10, 20].with_prev.collect() == [(None, 10), (Some(10), 20)]`
@get pub fn with_prev<S: Seq, T>(self: S) -> yields (?T, T):
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
/// cost: O(1) per step
/// empty: yields nothing; a single element yields `(x, x)`
/// order: left to right, then `(last, first)`
/// traps: never
/// example: `[1, 2, 3].cyclic_pairs().collect() == [(1, 2), (2, 3), (3, 1)]`
pub fn cyclic_pairs<S: Seq, T>(self: S) -> yields (T, T):
    var first_val: ?T = None
    var prev: ?T = None
    for x in self:
        if let p = prev: yield (p, x)
        else: first_val = x
        prev = x
    if let p = prev, let f = first_val:
        yield (p, f)

/// The pairs `(a.at(i), b.at(i))`, as many as the shorter sequence has.
/// Works on any `Indexed` sequences (`List`, `[D]T`, `Str`, `Range`, views). O(min(|a|, |b|)).
///
/// cost: O(min(|a|, |b|))
/// empty: yields nothing when either input is empty
/// order: position by position, stopping at the shorter
/// traps: never
/// example: `zip([1, 2, 3], "ab").collect() == [(1, 'a'), (2, 'b')]`
pub fn zip<A: Indexed, B: Indexed, T, U>(a: A, b: B) -> yields (T, U) = [(a.at(i), b.at(i)) for i in 0..min(a.len, b.len)]

/// The value `m` holds for `k`, after storing `v` there if it held none: `ids.get_or_insert(name, ids.len)`
/// gives each new name the next id. O(log n).
///
/// cost: O(log n)
/// traps: never
/// example: `var m: Map<Str, Int> = Map(); m.get_or_insert("a", 7) == 7 and m.get_or_insert("a", 9) == 7`
pub fn get_or_insert<K, V>(mut self: Map<K, V>, k: K, v: V) -> V:
    if not (k in self): self[k] = v
    self[k]

/// As `get_or_insert`, but the value is made by `f`, which runs only when `m` holds None
/// for `k`: `ids.get_or_insert_with(name, () => next_id())`. O(log n).
///
/// cost: O(log n), plus `f` once when `k` is absent
/// traps: never
/// example: `var m: Map<Int, Int> = Map(); m.get_or_insert_with(1, () => 5) == 5`
pub fn get_or_insert_with<K, V>(mut self: Map<K, V>, k: K, f: fn() -> V) -> V:
    if not (k in self): self[k] = f()
    self[k]

/// Replace the value under `k` by `f` of it, starting from `init` when `m` holds none:
/// `cnt.update(x, 0, c => c + 1)`. O(log n).
///
/// cost: O(log n), plus one call of `f`
/// traps: never
/// example: `var c: Map<Char, Int> = Map(); c.update('x', 0, v => v + 1); c['x'] == 1`
pub fn update<K, V>(mut self: Map<K, V>, k: K, init: V, f: fn(V) -> V):
    self[k] = f(self.get(k) ?? init)

/// Rotate `self` left by `k` in place (or only the subrange `range` when given):
/// what was at offset `k % len` comes first. Signed Euclidean `k`: negative `k`
/// rotates right. Works on any `IndexedMut` (`List<T>`, `[D]T`, `Str`). O(n), O(1) extra space.
///
/// cost: O(n), O(1) extra space
/// empty: does nothing
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
/// cost: O(n), O(1) extra space
/// empty: does nothing
/// traps: when `range` falls outside the keys
/// example: `var a = [1, 2, 3, 4]; a.rotate_right(1); a == [4, 1, 2, 3]`
pub fn rotate_right<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = None):
    rotate_left(mut self, -k, range:)

/// A new list rotated left by `k`: what was at offset `k % len` comes first.
/// Signed Euclidean `k`: negative `k` rotates right. Works on any `Seq`. O(n).
///
/// cost: O(n)
/// empty: `[]` for any `k`
/// order: a new `List`, keyed from 0
/// traps: never
/// example: `[1, 2, 3].rotated_left(-1) == [3, 1, 2]`
pub fn rotated_left<S: Seq, T>(self: S, k: Int) -> List<T>:
    let n = self.len
    if n <= 1: return [x for x in self]
    let s = k % n
    if s == 0: return [x for x in self]
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
/// cost: O(n)
/// empty: `[]` for any `k`
/// order: a new `List`, keyed from 0
/// traps: never
/// example: `[1, 2, 3].rotated_right(1) == [3, 1, 2]`
pub fn rotated_right<S: Seq, T>(self: S, k: Int) -> List<T>:
    rotated_left(self, -k)

/// Step `self` to the next arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted ascending, when it was the last one. Works on any `IndexedMut`. O(n), amortized O(1).
///
/// cost: O(n) worst case, amortized O(1) over all arrangements
/// empty: returns `false` and does nothing
/// ties: equal elements are one arrangement, so `[1, 1, 2]` has three
/// traps: never
/// example: `var a = [1, 2, 3]; a.next_permutation() and a == [1, 3, 2]`
pub fn next_permutation<S: IndexedMut>(mut self: S) -> Bool:
    var i = self.len - 2
    while i >= 0 and not (self.at(i) < self.at(i + 1)): i -= 1
    if i < 0:
        reverse(mut self)
        return false
    var j = self.len - 1
    while not (self.at(i) < self.at(j)): j -= 1
    self.swap_idx(i, j)
    var (l, r) = (i + 1, self.len - 1)
    while l < r:
        self.swap_idx(l, r)
        l += 1
        r -= 1
    true

/// Step `self` to the previous arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted descending, when it was the first (ascending) one. Works on any `IndexedMut`. O(n), amortized O(1).
///
/// cost: O(n) worst case, amortized O(1) over all arrangements
/// empty: returns `false` and does nothing
/// ties: equal elements are one arrangement
/// traps: never
/// example: `var a = [1, 3, 2]; a.prev_permutation() and a == [1, 2, 3]`
pub fn prev_permutation<S: IndexedMut>(mut self: S) -> Bool:
    var i = self.len - 2
    while i >= 0 and not (self.at(i + 1) < self.at(i)): i -= 1
    if i < 0:
        reverse(mut self)
        return false
    var j = self.len - 1
    while not (self.at(j) < self.at(i)): j -= 1
    self.swap_idx(i, j)
    var (l, r) = (i + 1, self.len - 1)
    while l < r:
        self.swap_idx(l, r)
        l += 1
        r -= 1
    true

/// The stretches of equal neighbours (or equal `by:` keys), in order:
/// each a slice of `self`: `"aabccc".runs()` walks `"aa"`, `"b"`, `"ccc"`, and
/// `[r.len for r in s.runs()]` is `[2, 1, 3]`. Each run's `.keys` is its range and
/// `r[r.keys.first!]` its key. O(1) to make, O(n) to walk.
///
/// cost: O(1) to make, O(n) to walk
/// empty: yields nothing
/// order: left to right; each run is a slice keeping the keys of `self`
/// traps: never
/// example: `[r.len for r in "aabccc".runs()] == [2, 1, 3]`
pub fn runs<S: Seq, T, K>(self: S, by: T -> K = x => x) -> yields Slice<S>:
    var out: List<(val: K, len: Int)> = []
    for x in self:
        let k = by(x)
        if out.len > 0 and out[out.len - 1].val == k:
            out[out.len - 1] = (val: k, len: out[out.len - 1].len + 1)
        else:
            out.push((val: k, len: 1))
    out

/// Bucket the keys of `self` by value: `b[k]` lists every `i` with `self[i] == k`, in key order.
/// The result is keyed by the value domain, so group by any domain-valued key with
/// `a.map(key).buckets()`; an optional table gives a `None` bucket (`b[Some(k)]`).
/// O(|D| + |K|) time and memory.
///
/// cost: O(|D| + |K|) time and memory
/// empty: every bucket is empty
/// order: each bucket lists keys of `self` in increasing order; the result is keyed by `K`
/// traps: never
/// example: `[1, 0, 1].buckets()` over values `0..2` is `[[1], [0, 2]]`
pub fn buckets<D, K: Domain>(self: [D]K) -> [K]List<D> = todo

/// The distinct values of a coordinate compression, in increasing order, indexed by their
/// own domain `D`. `xs[i]` is the value at index `i`, `xs.at(val)` the index of a value that
/// was compressed, and `xs.first_ge(val)` / `xs.last_le(val)` the nearest index (`?D`) for any value.
pub struct Compressed<D, T>(vals: [D]T):
    /// O(1). The value at index `i`; `slice` takes a `Range<D>`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let (domain C, xs) = compress([30, 10]); xs.get(C.first!) == 10`
    pub fn get(self, i: D) -> T = vals[i]

    /// The compressed values over a range `r` of `xs.D`, as a list.
    ///
    /// cost: O(r.len)
    /// empty: `[]` for an empty `r`
    /// order: increasing values; a new `List`, keyed from 0
    /// traps: never
    /// example: `let (domain C, xs) = compress([3, 1, 2]); xs.slice(C.all) == [1, 2, 3]`
    pub fn slice(self, r: Range<D>) -> List<T> = [vals[i] for i in r]

    /// How many distinct values there are.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `compress([5, 5, 7]).1.len == 2`
    @get pub fn len(self) -> Int = todo

    /// The index of `val`, which must be one of the compressed values.
    ///
    /// cost: O(log n)
    /// ties: none: values are distinct
    /// traps: when `val` was not compressed
    /// example: `let (domain C, xs) = compress([30, 10]); xs.at(30) == C.last!`
    pub fn at(self, val: T) -> D:
        let i = vals.first_ge(val)
        assert i != None and vals[i!] == val  // `at` is for a value that was compressed
        i!

    /// The index of `val` if it was compressed, or `None`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `compress([30, 10]).1.try_at(20) == None`
    pub fn try_at(self, val: T) -> ?D:
        if let i = vals.first_ge(val), vals[i] == val: i else: None

    /// Whether `val` is one of the compressed values.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `compress([30, 10]).1.contains(10) == true`
    pub fn contains(self, val: T) -> Bool = self.try_at(val) != None

    /// The first index whose value is `>= val`, or `None`.
    ///
    /// cost: O(log n)
    /// ties: none: values are distinct
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.first_ge(20) == Some(C.last!)`
    pub fn first_ge(self, val: T) -> ?D = vals.first_ge(val)

    /// The first index whose value is `> val`, or `None`.
    ///
    /// cost: O(log n)
    /// ties: none: values are distinct
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.first_gt(30) == None`
    pub fn first_gt(self, val: T) -> ?D = vals.first_gt(val)

    /// The last index whose value is `<= val`, or `None`.
    ///
    /// cost: O(log n)
    /// ties: none: values are distinct
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.last_le(20) == Some(C.first!)`
    pub fn last_le(self, val: T) -> ?D = vals.last_le(val)

    /// The last index whose value is `< val`, or `None`.
    ///
    /// cost: O(log n)
    /// ties: none: values are distinct
    /// traps: never
    /// example: `let (domain C, xs) = compress([10, 30]); xs.last_lt(10) == None`
    pub fn last_lt(self, val: T) -> ?D = vals.last_lt(val)

/// Coordinate compression: `let (domain C, xs) = compress(all)` sorts the values, drops the
/// duplicates and binds a fresh domain `C` for them, so an array over them is `[C]Int` and an
/// index of one compression cannot be used in another. Works on any `Seq`. O(n log n).
///
/// cost: O(n log n)
/// empty: binds an empty domain `C`
/// order: increasing values, indexed by the fresh domain `C`
/// traps: never
/// example: `let (domain C, xs) = compress([30, 10, 30]); xs.len == 2 and xs.at(30) == C.last!`
pub fn compress<S: Seq, T>(all: S) -> (domain C, Compressed<C, T>) = todo

/// Split a range at cuts: `let (domain P, piece) = D.pieces(cuts: cs)` binds a fresh domain of the
/// non-empty pieces between consecutive cuts (the range's own ends always count; repeats drop),
/// with `piece[p]` the `Range<D>` that piece covers, in order. It is coordinate compression for
/// intervals: a segment tree over `P` stores `piece[p].len`, and no +1/-1 converts back.
/// O(k log k) for k cuts.
///
/// cost: O(k log k) for k cuts
/// empty: an empty range binds an empty `P`
/// order: pieces left to right; `piece` is keyed by the fresh domain `P`
/// traps: when a cut lies outside the range
/// example: `let (domain P, piece) = (0..10).pieces(cuts: [3, 7]); [piece[p].len for p in P] == [3, 4, 3]`
pub fn pieces<D, S: Seq>(self: Range<D>, cuts: S) -> (domain P, [P]Range<D>) = todo

/// The range of pieces exactly covering `r`, whose ends must be among the cuts the pieces were
/// made from (traps otherwise). `t.apply(piece.covering(a..=b), x)`. O(log k).
///
/// cost: O(log k)
/// empty: an empty `r` gives an empty range of pieces
/// traps: when an end of `r` is not one of the cuts
/// example: with the pieces of `(0..10).pieces(cuts: [3, 7])`, `piece.covering(3..10).len == 2`
/// order: pieces left to right, keyed by `P`
pub fn covering<P, D>(self: [P]Range<D>, r: Range<D>) -> Range<P> = todo

/// The first `f(x)` that is not `None`, stopping there; `None` if every one is.
/// O(1) per element walked.
///
/// cost: O(1) per element walked, stopping at the first `Some`
/// empty: `None`
/// ties: the first in sequence order
/// traps: never
/// example: `[1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None) == Some(40)`
pub fn first_some<S: Seq, T, U>(self: S, f: T -> ?U) -> ?U:
    for x in self:
        let y = f(x)
        if y != None: return y
    None

/// The first element satisfying `p`, `None` if None does; stops there. `(1..=m).find(ok)` is the
/// least `ok` integer. O(position found).
///
/// cost: O(position found) calls of `p`
/// empty: `None`
/// ties: the first in sequence order
/// traps: never
/// example: `(1..=10).find(x => x * x > 20) == Some(5)`
pub fn find<S: Seq, T>(self: S, p: T -> Bool) -> ?T:
    for x in self:
        if p(x): return Some(x)
    None

/// Gives the items of a list a fresh domain, so arrays over them are index-safe:
/// `let (domain Arc, arcs) = [...].with_domain()`. Moves the list, O(1).
///
/// cost: O(1); moves the list
/// empty: binds an empty domain
/// order: the list's order, keyed by the fresh domain
/// traps: never
/// example: `let (domain I, xs) = [7, 8].with_domain(); xs[I.first!] == 7`
pub fn with_domain<T>(self: List<T>) -> (domain I, [I]T) = todo

/// The cycle of a permutation through `from`: `from`, `perm[from]`, ... until it
/// returns to `from`. O(1) to make, O(cycle length) to walk.
///
/// cost: O(1) to make, O(1) per step
/// order: `from`, `self[from]`, ... stopping before `from` comes back
/// traps: never; a non-permutation that never returns to `from` walks forever
/// example: `[1, 2, 0].cycle(from: 0).collect() == [0, 1, 2]`
/// empty: never empty: yields at least `from`
pub fn cycle<D>(self: [D]D, from: D) -> yields D:
    var x = from
    while true:
        yield x
        x = self[x]
        if x == from: return

// ---- Range boundary cuts, keyed sorted lookup, ordered Map 4-lookups, and List/Array helpers

/// Left boundary cut of a range (valid even when empty).
///
/// cost: O(1)
/// empty: the same cut as `r`
/// traps: never
/// example: `(2..5).l.r == Some(2)`
@get pub fn l<D>(self: Range<D>) -> Cut<D> = self.start

/// Right boundary cut of a range (valid even when empty).
///
/// cost: O(1)
/// empty: the same cut as `l`
/// traps: never
/// example: `(2..5).r.l == Some(4)`
@get pub fn r<D>(self: Range<D>) -> Cut<D> = self.end

/// O(1). `(r.l..r.mid, r.mid..r.r)`: two non-empty halves, or `None` when `self.len <= 1`, so
/// `let (left, right) = r.halve() else: base` is the divide-and-conquer base case.
///
/// cost: O(1)
/// empty: `None`, as for one element
/// traps: never
/// example: `(0..5).halve() == Some((0..2, 2..5))`
/// order: left half first
pub fn halve<D>(self: Range<D>) -> ?(Range<D>, Range<D>) = todo

/// O(1). The middle cut: ⌊len/2⌋ elements before it; `r.l` when empty. Its `.r` is the
/// middle element only when `r` is non-empty, so guard with `if r.is_empty` first.
///
/// cost: O(1)
/// empty: `r.l`
/// traps: never
/// example: `(0..5).mid.r == Some(2)`
@get pub fn mid<D>(self: Range<D>) -> Cut<D> = todo

/// O(1). The positions in both ranges.
///
/// cost: O(1)
/// empty: an empty range when they do not meet
/// traps: never
/// example: `(0..5).clip(3..9) == 3..5`
/// order: increasing, a sub-range of both
pub fn clip<D>(self: Range<D>, other: Range<D>) -> Range<D>:
    if self.is_empty: return self
    if other.is_empty: return other
    let lo = if self.first! < other.first!: other.first! else: self.first!
    let hi = if self.last! < other.last!: self.last! else: other.last!
    if hi < lo: lo..lo else: lo..=hi

/// O(log n) calls to `ok`. The first element where `ok`, false then true along `r`, holds;
/// `None` when it never does. Checked builds check the probes agree (R09.2.3).
///
/// cost: O(log n) calls of `ok`
/// empty: `None`
/// traps: never in release; debug traps when `ok` is not false-then-true at the answer
/// example: `(0..100).first_true(x => x * x >= 50) == Some(8)`
/// ties: none: a monotone `ok` has one first true position
pub fn first_true<D>(self: Range<D>, ok: D -> Bool) -> ?D:
    var lo = 0
    var hi = self.len
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if ok(self[m]): hi = m
        else: lo = m + 1
    if lo < self.len: Some(self[lo]) else: None

/// O(log n) calls to `ok`. The last element where `ok`, true then false along `r`, holds;
/// `None` when it never does. Checked builds check the probes agree (R09.2.3).
///
/// cost: O(log n) calls of `ok`
/// empty: `None`
/// traps: never in release; debug traps when `ok` is not true-then-false at the answer
/// example: `(0..100).last_true(x => x * x <= 50) == Some(7)`
/// ties: none: a monotone `ok` has one last true position
pub fn last_true<D>(self: Range<D>, ok: D -> Bool) -> ?D:
    var lo = 0
    var hi = self.len
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if ok(self[m]): lo = m + 1
        else: hi = m
    if lo > 0: Some(self[lo - 1]) else: None

/// O(log n) calls to `ok`. The cut where `ok` flips: false then true along `r` by default
/// (`r.l` when it always holds, `r.r` when it never does), true then false with `last: true`
/// (`r.r` when it always holds). `first_true(ok)` is `.r` of it; `last_true(ok)` is
/// `bsearch(ok, last: true).l`.
///
/// cost: O(log n) calls of `ok`
/// empty: `r.l`
/// traps: never
/// example: `(0..100).bsearch(x => x * x >= 50).r == 8`
pub fn bsearch<D>(self: Range<D>, ok: D -> Bool, last: Bool = false) -> Cut<D>:
    let n = self.len
    var lo = 0
    var hi = n
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if ok(self[m]) != last: hi = m
        else: lo = m + 1
    if lo < n: (self[lo]..self[lo]).start else: self.end

/// The two ends of a range as offsets, `(lo, hi)` with `lo <= hi`: what a range query walks.
/// A range written backwards, `P(3)..P(1)`, stops the program here: no query silently answers
/// as if it were empty. `l..l` is in order and empty.
///
/// cost: O(1)
/// empty: `(lo, lo)`
/// traps: when the range is written backwards
/// example: `(2..5).ends == (2, 5)`
@get pub fn ends<D>(self: Range<D>) -> (Int, Int):
    let lo = self.start.ord
    let hi = self.end.ord
    assert lo <= hi, "the range is written backwards"
    (lo, hi)

/// First index in sorted `self` with `by(self[i]) > target`, or `None`.
///
/// cost: O(log n)
/// empty: `None`
/// ties: the first key past every equal value
/// traps: never; unsorted input gives an unspecified key
/// example: `[1, 3, 3, 5].first_gt(3) == Some(3)`
pub fn first_gt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    var (lo, hi) = (0, D.K)
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if by(self[D.nth(m)]) > target: hi = m else: lo = m + 1
    if lo < D.K: Some(D.nth(lo)) else: None

/// Last index in sorted `self` with `by(self[i]) < target`, or `None`.
///
/// cost: O(log n)
/// empty: `None`
/// ties: the last key before every equal value
/// traps: never; unsorted input gives an unspecified key
/// example: `[1, 3, 3, 5].last_lt(3) == Some(0)`
pub fn last_lt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    var (lo, hi) = (0, D.K)
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if by(self[D.nth(m)]) < target: lo = m + 1 else: hi = m
    if lo > 0: Some(D.nth(lo - 1)) else: None

// ---- Permutations, monotone stack, and sliding-window extrema

/// Inverse of bijection `self: [I]V`: `inv[self[i]] == i` for all `i in I`.
/// Traps if `self` is not a bijection. O(|I|).
///
/// cost: O(|I|)
/// empty: an empty array
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
/// cost: O(|A|)
/// empty: an empty array
/// order: keyed by `A`
/// traps: never
/// example: `compose([1, 0], [5, 6]) == [6, 5]`
pub fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C = [A] a => g[f[a]]

/// Disjoint cycle decomposition of permutation `p`, each cycle starting at its smallest element.
///
/// cost: O(|V|)
/// empty: `[]`
/// order: cycles by smallest element; each starts there and follows `p`
/// traps: never in release; debug traps when `p` is not a permutation
/// example: `perm_cycles([1, 0, 2]) == [[0, 1], [2]]`
pub fn perm_cycles<V>(p: [V]V) -> List<List<V>>:
    var vis: [V]Bool = false
    var out: List<List<V>> = []
    for start in V:
        if vis[start]: continue
        var cyc: List<V> = []
        var u = start
        while not vis[u]:
            vis[u] = true
            cyc.push(u)
            u = p[u]
        out.push(cyc)
    out

/// `k`-th iterate of permutation `p` (`k >= 0`), computed via cycle decomposition.
///
/// cost: O(|V|)
/// empty: an empty array
/// order: keyed by `V`
/// traps: when `k < 0`
/// example: `perm_pow([1, 2, 0], 2) == [2, 0, 1]`
pub fn perm_pow<V>(p: [V]V, k: Int) -> [V]V:
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
/// cost: O(n log n)
/// empty: `0`
/// ties: equal values are not an inversion
/// traps: never
/// example: `inversions([3, 1, 2]) == 2`
pub fn inversions<S: Seq, T>(a: S) -> Int:
    var cur: List<T> = [x for x in a]
    let n = cur.len
    var tmp = cur.copy()
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
///
/// cost: O(1)
/// traps: never
/// example: `var a = [1, 2]; a.swap_at(0, 1); a == [2, 1]`
/// empty: no key exists, so no call is possible
pub fn swap_at<D, T>(mut self: [D]T, i: D, j: D):
    if i == j: return
    let t = self[i]
    self[i] = self[j]
    self[j] = t

/// `self` cut or padded with `fill` to length `n`: `a.resized(size, Z(0))` instead of
/// `[if i < a.len: a[i] else: Z(0) for i in 0..size]`. Works on any `Indexed` sequence. O(n).
///
/// cost: O(n)
/// empty: `n` copies of `fill`
/// order: a new `List`, keyed from 0
/// traps: when `n < 0`
/// example: `[1, 2, 3].resized(5, 0) == [1, 2, 3, 0, 0]`
pub fn resized<S: Indexed, T>(self: S, n: Int, fill: T) -> List<T> = [if i < self.len: self.nth(i) else: fill for i in 0..n]

/// O(1). Pop the last element if `ok` holds for it, else leave `self` alone and give `None`.
/// A monotonic stack is `while let top = st.pop_if(t => t.h >= h): ...`.
///
/// cost: O(1)
/// empty: `None`
/// traps: never
/// example: `var s = [1, 5]; s.pop_if(x => x > 3) == Some(5)`
pub fn pop_if<T>(mut self: List<T>, ok: T -> Bool) -> ?T = if self.len > 0 and ok(self[self.len - 1]): self.pop() else: None

/// O(popped). Pop from the back while `ok` holds for the last element; the popped
/// elements, last first. The back twin of `Deque.pop_front_while`.
///
/// cost: O(popped)
/// empty: `[]`
/// order: last element first
/// traps: never
/// example: `var s = [1, 4, 5]; s.pop_while(x => x > 3) == [5, 4] and s == [1]`
pub fn pop_while<T>(mut self: List<T>, ok: T -> Bool) -> List<T>:
    var out: List<T> = []
    while self.len > 0 and ok(self[self.len - 1]): out.push(self.pop()!)
    out

/// O(n), allocates. A separate copy: `let b = a` moves, `let b = a.clone()` copies.
///
/// cost: O(size of the value), allocates for heap values
/// traps: never
/// example: `let a = [1]; var b = a.clone(); b.push(2); a == [1]`
pub fn clone<T>(self: T) -> T = self.copy()

/// Move all elements out of `self` in O(1), leaving it empty.
///
/// cost: O(1)
/// empty: `[]`
/// order: unchanged; `self` is left empty
/// traps: never
/// example: `var a = [1, 2]; a.take_all() == [1, 2] and a == []`
pub fn take_all<T>(mut self: List<T>) -> List<T> = self.take()

/// Insert `x` at index `i` (`0 <= i <= self.len`), shifting elements `i..` right by one.
///
/// cost: O(n - i)
/// traps: when `i` is not in `0..=self.len`
/// example: `var a = [1, 3]; a.insert(1, 2); a == [1, 2, 3]`
/// empty: `i` must be 0
pub fn insert<T>(mut self: List<T>, i: Int, x: T):
    assert 0 <= i and i <= self.len
    self.push(x)
    var j = self.len - 1
    while j > i:
        self.swap(j - 1, j)
        j -= 1

/// O(n). Remove and return the element at index `i` (`0 <= i < self.len`), shifting `i + 1..` left.
///
/// cost: O(n - i)
/// traps: when `i` is not in `0..self.len`
/// example: `var a = [1, 2, 3]; a.remove(1) == 2 and a == [1, 3]`
/// empty: always traps
pub fn remove<T>(mut self: List<T>, i: Int) -> T:
    assert 0 <= i and i < self.len
    for j in i..self.len - 1:
        self.swap(j, j + 1)
    self.pop()!

// =============================================================================
// Spec surface (`= todo` where not yet implemented)
// =============================================================================

// --- Lazy views, slicing, and stream / combinator generators (W4, W6) ---

/// O(1) to make. The view `s.filter(p)`: walks `s`, skipping elements that fail `p`, with
/// `s`'s keys. A lent value (R07.1.5).
pub struct Filtered<S>():
    /// O(1). The source sequence this view filters.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a = [1, 2]; a.filter(x => x > 1).src == a`
    @get pub fn src(self) -> S = todo

/// O(1) to make. The view `s.map(f)`: `f` of each element, with `s`'s keys; Indexed when `s` is.
pub struct Mapped<S, U>():
    /// O(1). The source sequence this view maps.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a = [1, 2]; a.map(x => x * 2).src == a`
    @get pub fn src(self) -> S = todo

/// O(1) to make. The view `s.rev`: `s` walked backwards, keys unchanged.
pub struct Rev<S>():
    /// O(1). The source sequence this view reverses.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a = [1, 2]; a.rev.src == a`
    @get pub fn src(self) -> S = todo

/// O(1) to make. The view `s.items`: `(key, element)` pairs in `s`'s order.
pub struct Items<S>():
    /// O(1). The source sequence whose items this view walks.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a = [1, 2]; a.items.src == a`
    @get pub fn src(self) -> S = todo

/// O(1) to make. `s[r]`: a borrowed view of `s` over the range `r`. Every slice keeps its
/// source's keys: `a[l..=r][i]` is `a[i]` and `i` must lie in `l..=r`; for a `List` or `Str`,
/// `s[2..5][3]` is `s[3]` and `s[2..5][0]` traps. `w.first`, `w.nth(k)` and `w.rebase()` give
/// offsets. Writable through a `var` source; the source's length can't change while it is alive.
/// See `spec/core/04b-keys.md`.
pub struct Slice<S>():
    /// O(1). The slice's keys: a sub-range of the source's keys.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[5, 6, 7][1..].keys == 1..3`
    /// order: increasing
    @get pub fn keys<K>(self) -> Range<K> = todo
    ///
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3][1..].len == 2`
    @get pub fn len(self) -> Int = todo
    ///
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2][2..].is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo
    ///
    ///
    /// cost: O(1) per step
    /// empty: yields nothing
    /// order: the slice's keys, increasing
    /// traps: never
    /// example: `[5, 6, 7][1..].iter().collect() == [1, 2]`
    pub fn iter(self) -> yields Int = todo

/// O(1). The view without its first `k` elements, clamped (traps if `k < 0`). Consuming a view
/// from the front is `w = w.drop(1)`.
///
/// cost: O(1)
/// empty: an empty view
/// order: keeps the keys of `self`
/// traps: when `k < 0`
/// example: `[1, 2, 3].drop(1).collect() == [2, 3]`
pub fn drop<S: Indexed>(self: S, k: Int = 1) -> Slice<S> = todo

/// O(1). The view without its last `k` elements, clamped (traps if `k < 0`).
///
/// cost: O(1)
/// empty: an empty view
/// order: keeps the keys of `self`
/// traps: when `k < 0`
/// example: `[1, 2, 3].drop_last(2).collect() == [1]`
pub fn drop_last<S: Indexed>(self: S, k: Int = 1) -> Slice<S> = todo

/// O(1). The first `min(k, len)` elements, as a view (traps if `k < 0`).
///
/// cost: O(1)
/// empty: an empty view
/// order: keeps the keys of `self`
/// traps: when `k < 0`
/// example: `[1, 2, 3].take(5).collect() == [1, 2, 3]`
pub fn take<S: Indexed>(self: S, k: Int) -> Slice<S> = todo

/// O(1). The last `min(k, len)` elements, as a view (traps if `k < 0`).
///
/// cost: O(1)
/// empty: an empty view
/// order: keeps the keys of `self`
/// traps: when `k < 0`
/// example: `[1, 2, 3].take_last(2).collect() == [2, 3]`
pub fn take_last<S: Indexed>(self: S, k: Int) -> Slice<S> = todo

/// O(1). The element `k` steps from the start, 0-based whatever the keys: `s[l..].nth(k)` is
/// `s[l + k]`, `w.nth(0)` is `w.first!`. Traps when `k` is outside `0..len`.
///
/// cost: O(1)
/// empty: traps for every `k`
/// traps: when `k` is not in `0..self.len`
/// example: `[5, 6, 7][1..].nth(0) == 6`
pub fn nth<S: Indexed, T>(self: S, k: Int) -> T = self.at(self.keys[k])

/// O(1). `self` keyed by a fresh domain `0..len` of its own (`let t = w.rebase`, keys `t.Idx`),
/// for 0-based arithmetic on a piece; its keys can't be mixed with the source's.
///
/// cost: O(1)
/// empty: an empty view over an empty domain
/// order: same elements, keyed `0..len` by a fresh domain
/// traps: never
/// example: `[5, 6, 7][1..].rebase.nth(1) == 7`
@get pub fn rebase<S: Indexed>(self: S) -> Rebased<S> = todo

/// The view `rebase` returns.
pub struct Rebased<S>(src: S)

/// O(1). The same cells keyed by `keys`, which must have the same length (traps otherwise):
/// a table computed on offsets handed back in the caller's keys, `z.rekey(s.keys)`.
///
/// cost: O(1)
/// empty: valid only with empty `keys`
/// order: same cells, keyed by `keys`
/// traps: when `keys.len != self.len`
/// example: `[4, 5].rekey(10..12)[11] == 5`
pub fn rekey<D, K, T>(self: [D]T, keys: Range<K>) -> [K]T = todo

/// O(n), allocates. The end of a lazy chain, materialized into the type the context expects
/// (`List`, `Set`, `Map`, `Str`, `[D]T`), else a `List`: `a.filter(p).map(f).collect()`.
/// The same operation as `List(s)`; the postfix form is for chains.
///
/// cost: O(n), allocates
/// empty: an empty container
/// order: input order; the container the context expects, else a `List`
/// traps: never
/// example: `(1..4).map(x => x * x).collect() == [1, 4, 9]`
pub fn collect<S: Seq, T>(self: S) -> List<T>:
    var out: List<T> = []
    for x in self: out.push(x)
    out

/// `self` with keys attached, so it is `Keyed`: walking it still gives values, `.items`
/// gives `(key, x)`, and `argmax`/`find` answer with keys. By default the k-th element gets
/// the k-th key of `keys` (`0..` is a counter: `for (i, x) in g.keyed().items`); with `by:`
/// each key is `by(x)` (`dfs(s).keyed(by: v => v)`). Passing both is an error. O(1) to make.
///
/// cost: O(1) to make, O(1) per step
/// empty: an empty view
/// order: input order, with the k-th key of `keys` or `by(x)`
/// traps: when both `keys` and `by` are given; when `keys` runs out first
/// example: `"ab".keyed().items.collect() == [(0, 'a'), (1, 'b')]`
pub fn keyed<S: Seq, I: Indexed, T, K>(self: S, keys: I = 0.., by: ?(T -> K) = None) -> Keys<S, I, K> = todo

/// The view `keyed` returns.
pub struct Keys<S, I, K>(src: S, keys: I, by: ?K)

/// O(1) per step. Yield leading elements while `keep(x)` holds, stopping at the first failure.
///
/// cost: O(1) per step
/// empty: yields nothing
/// order: input order
/// traps: never
/// example: `[1, 2, 5, 1].take_while(x => x < 3).collect() == [1, 2]`
pub fn take_while<S: Seq, T>(self: S, keep: T -> Bool) -> yields T:
    for x in self:
        if not keep(x): return
        yield x

/// O(1) per step. Skip leading elements while `drop(x)` holds, then yield the rest.
///
/// cost: O(1) per step
/// empty: yields nothing
/// order: input order
/// traps: never
/// example: `[1, 2, 5, 1].drop_while(x => x < 3).collect() == [5, 1]`
pub fn drop_while<S: Seq, T>(self: S, drop: T -> Bool) -> yields T:
    var dropping = true
    for x in self:
        if dropping and drop(x): continue
        dropping = false
        yield x

/// O(1) per step. Lazily yield `f(x)` for each element where it is not `None`:
/// `dag[c].filter_map(a => best(a.to)).max()`.
///
/// cost: O(1) per step
/// empty: yields nothing
/// order: input order
/// traps: never
/// example: `[1, 2, 3].filter_map(x => if x != 2: Some(x * 10) else: None).collect() == [10, 30]`
pub fn filter_map<S: Seq, T, U>(self: S, f: T -> ?U) -> yields U:
    for x in self:
        if let y = f(x): yield y

/// O(1) per yielded element. Lazily map each element to a sequence and yield all inner elements in order.
///
/// cost: O(1) per yielded element
/// empty: yields nothing
/// order: outer order, then each inner sequence's order
/// traps: never
/// example: `[1, 2].flat_map(x => 0..x).collect() == [0, 0, 1]`
pub fn flat_map<S: Seq, T, U: Seq, V>(self: S, f: T -> U) -> yields V:
    for x in self:
        for y in f(x): yield y

/// O(1) per yielded element. Lazily concatenate a sequence of sequences.
///
/// cost: O(1) per yielded element
/// empty: yields nothing
/// order: outer order, then inner order
/// traps: never
/// example: `[[1], [], [2, 3]].flatten().collect() == [1, 2, 3]`
pub fn flatten<S: Seq, U: Seq, T>(self: S) -> yields T:
    for xs in self:
        for x in xs: yield x

/// O(1) per step. Yield the `Some` values of a sequence of `?T`, skipping `None`.
///
/// cost: O(1) per step
/// empty: yields nothing
/// order: input order
/// traps: never
/// example: `[Some(1), None, Some(3)].present().collect() == [1, 3]`
pub fn present<S: Seq, T>(self: S) -> yields T:
    for x in self:
        if let y = x: yield y

/// O(n), O(n) memory. Unwrap every `?T` into a `List<T>`, or return `None` if any element is `None`.
///
/// cost: O(n), O(n) memory
/// empty: `Some([])`
/// order: input order
/// traps: never
/// example: `[Some(1), None].all_some() == None`
pub fn all_some<S: Seq, T>(self: S) -> ?List<T>:
    var out: List<T> = []
    for x in self:
        if let y = x: out.push(y)
        else: return None
    Some(out)

/// O(1) per step. Lazily concatenate `a` then `b` without copying.
///
/// cost: O(1) per step
/// empty: the other sequence
/// order: all of `a`, then all of `b`
/// traps: never
/// example: `chain([1], [2, 3]).collect() == [1, 2, 3]`
pub fn chain<A: Seq, B: Seq, T>(a: A, b: B) -> yields T:
    for x in a: yield x
    for x in b: yield x

/// O(1) per step. Cartesian product `(x, y)` for `x in a`, `y in b` in lexicographic order.
///
/// cost: O(1) per step, O(|a| · |b|) to walk
/// empty: yields nothing when either is empty
/// order: lexicographic, `a` outer
/// traps: never
/// example: `product([1, 2], "xy").count() == 4`
pub fn product<A: Seq, B: Indexed, T, U>(a: A, b: B) -> yields (T, U):
    for x in a:
        for y in b: yield (x, y)

/// O(1) per step. One-shot stream `from, step(from), step(step(from)), ...` until `None`.
///
/// cost: O(1) per step plus one call of `step`
/// empty: yields nothing when `from` is `None`
/// order: `from`, `step(from)`, ... until `None`
/// traps: never
/// example: `iterate(Some(1), x => if x < 8: Some(2 * x) else: None).collect() == [1, 2, 4, 8]`
pub fn iterate<T>(from: ?T, step: T -> ?T) -> yields T:
    var cur = from
    while let x = cur:
        yield x
        cur = step(x)

/// O(1) per step. Every element paired with the next one, ending with `(last, None)`.
///
/// cost: O(1) per step
/// empty: yields nothing
/// order: left to right, ending `(last, None)`
/// traps: never
/// example: `[1, 2].with_next().collect() == [(1, Some(2)), (2, None)]`
pub fn with_next<S: Seq, T>(self: S) -> yields (T, ?T):
    var prev: ?T = None
    for x in self:
        if let p = prev: yield (p, Some(x))
        prev = Some(x)
    if let p = prev: yield (p, None)

/// O(1) per step. Zero-alloc zipped view of equal-length sequences; traps if `a.len != b.len`.
///
/// cost: O(1) per step
/// empty: yields nothing when both are empty
/// order: position by position
/// traps: when `a.len != b.len`
/// example: `zip_exact([1, 2], "ab").collect() == [(1, 'a'), (2, 'b')]`
pub fn zip_exact<A: Indexed, B: Indexed, T, U>(a: A, b: B) -> yields (T, U):
    assert a.len == b.len
    for i in 0..a.len: yield (a.nth(i), b.nth(i))

/// O(n). Non-overlapping length-`k` chunks, padding the last chunk with `pad` when shorter.
///
/// cost: O(k) per chunk
/// empty: yields nothing
/// order: left to right
/// traps: when `k < 1`
/// example: `[1, 2, 3].chunks_padded(2, 0).collect() == [[1, 2], [3, 0]]`
pub fn chunks_padded<S: Indexed, T>(self: S, k: Int, pad: T) -> yields List<T> = todo

// --- Reductions, folds, frequencies, and selection ---

/// O(n). Seedless left fold returning `None` on empty (never traps).
///
/// cost: O(n) calls of `f`
/// empty: `None`
/// traps: never
/// example: `[3, 4].reduce((a, b) => a * b) == Some(12)`
pub fn reduce<S: Seq, T>(self: S, f: (T, T) -> T) -> ?T:
    var acc: ?T = None
    for x in self:
        if let a = acc: acc = Some(f(a, x))
        else: acc = Some(x)
    acc

/// O(n). Seeded left fold into an arbitrary accumulator `Acc`.
///
/// cost: O(n) calls of `f`
/// empty: `init`
/// traps: never
/// example: `[1, 2, 3].fold((s, x) => s * 10 + x, 0) == 123`
pub fn fold<S: Seq, T, Acc>(self: S, f: (Acc, T) -> Acc, init: Acc) -> Acc:
    var acc = init
    for x in self: acc = f(acc, x)
    acc

/// O(n). Maximum element (under `by:`) together with its multiplicity, or `None` when empty.
///
/// cost: O(n)
/// empty: `None`
/// ties: `val` is the first maximal element; `count` counts every element with an equal key
/// traps: never
/// example: `[3, 1, 3].max_count() == Some((val: 3, count: 2))`
pub fn max_count<S: Seq, T, K>(self: S, by: T -> K = x => x) -> ?(val: T, count: Int):
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

/// O(n). Minimum element (under `by:`) together with its multiplicity, or `None` when empty.
///
/// cost: O(n)
/// empty: `None`
/// ties: `val` is the first minimal element; `count` counts every element with an equal key
/// traps: never
/// example: `[1, 2, 1].min_count() == Some((val: 1, count: 2))`
pub fn min_count<S: Seq, T, K>(self: S, by: T -> K = x => x) -> ?(val: T, count: Int):
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

/// The value held by more than half the elements, with its count, or `None`. Only `eq` is
/// asked, never order or hashing, so it works with an equality oracle (Boyer-Moore: one
/// pass to find the candidate, one to count it). O(n) `eq` calls, O(1) memory.
///
/// cost: O(n) calls of `eq`, O(1) memory
/// empty: `None`
/// traps: never
/// example: `[2, 1, 2].majority() == Some((val: 2, count: 2))`
pub fn majority<S: Seq, T>(self: S, eq: (T, T) -> Bool = (a, b) => a == b) -> ?(val: T, count: Int):
    var cand: ?T = None
    var votes = 0
    var n = 0
    for x in self:
        n += 1
        if votes == 0:
            cand = Some(x)
            votes = 1
        elif eq(cand!, x): votes += 1
        else: votes -= 1
    let c = cand else: return None
    var count = 0
    for x in self:
        if eq(c, x): count += 1
    if 2 * count > n: Some((val: c, count:)) else: None

/// O(n) expected, O(distinct) memory. Frequency map of keys `by(x)` across `self`.
///
/// cost: O(n) expected, O(distinct) memory
/// empty: an empty map
/// traps: never
/// example: `"abca".tally()['a'] == 2`
pub fn tally<S: Seq, T, K>(self: S, by: T -> K = x => x) -> HashMap<K, Int>:
    var out = HashMap<K, Int>()
    for x in self:
        let k = by(x)
        let c = out.get(k)
        out[k] = if c == None: 1 else: c! + 1
    out

/// O(n) expected, O(n) memory. Group elements by an arbitrary hashable key; `of:` keeps a projection of each element (`pairs.group(by: .0, of: .1)`). When the key is a domain, `a.map(key).buckets()` keys the result by that domain instead.
///
/// cost: O(n) expected, O(n) memory
/// empty: an empty map
/// order: each group keeps input order
/// traps: never
/// example: `[(1, 'a'), (2, 'b'), (1, 'c')].group(by: .0, of: .1)[1] == ['a', 'c']`
pub fn group<S: Seq, T, K, U>(self: S, by: T -> K, of: T -> U = x => x) -> HashMap<K, List<U>>:
    var out = HashMap<K, List<U>>()
    for x in self:
        let k = by(x)
        if out.get(k) == None: out[k] = []
        out[k].push(of(x))
    out

/// O(n) expected, O(k) memory. `k` smallest elements under `by:` (unordered; ties to earlier index).
///
/// cost: O(n) expected, O(k) memory
/// empty: `[]`
/// ties: earlier elements win
/// order: unspecified; sort it when order matters
/// traps: when `k < 0`; `k > n` gives every element
/// example: `[5, 1, 4, 2].min_k(2).sorted() == [1, 2]`
pub fn min_k<S: Seq, T, K>(self: S, k: Int, by: T -> K = x => x) -> List<T>:
    var ps: List<(K, Int, T)> = []
    for x in self: ps.push((by(x), ps.len, x))
    if k < ps.len and k > 0: select(mut ps, k - 1, by: p => (p.0, p.1))
    [ps[i].2 for i in 0..(if k < ps.len: k else: ps.len)]

/// O(n) expected, O(k) memory. `k` largest elements under `by:` (unordered; ties to earlier index).
///
/// cost: O(n) expected, O(k) memory
/// empty: `[]`
/// ties: earlier elements win
/// order: unspecified; sort it when order matters
/// traps: when `k < 0`; `k > n` gives every element
/// example: `[5, 1, 4, 2].max_k(2).sorted() == [4, 5]`
pub fn max_k<S: Seq, T, K>(self: S, k: Int, by: T -> K = x => x) -> List<T>:
    var ps: List<(K, Int, T)> = []
    for x in self: ps.push((by(x), ps.len, x))
    if k < ps.len and k > 0: select(mut ps, k - 1, cmp: (p, q) => q.0 < p.0 or (p.0 == q.0 and p.1 < q.1))
    [ps[i].2 for i in 0..(if k < ps.len: k else: ps.len)]

/// O(n). Unstable in-place partition moving `keep(x)` elements to `0..mid`; returns `mid`.
///
/// cost: O(n), O(1) extra space
/// empty: returns `0`
/// order: kept elements in `0..mid`, the rest after, each side in unspecified order
/// traps: never
/// example: `var a = [1, 2, 3, 4]; a.partition_in_place(x => x % 2 == 0) == 2`
pub fn partition_in_place<S: IndexedMut, T>(mut self: S, keep: T -> Bool) -> Int:
    var mid = 0
    for i in 0..self.len:
        if keep(self.at(i)):
            self.swap_idx(mid, i)
            mid += 1
    mid

/// O(n), O(n) memory. Stable in-place partition preserving relative order in both halves; returns `mid`.
///
/// cost: O(n), O(n) memory
/// empty: returns `0`
/// order: kept elements in `0..mid`, the rest after, each side in input order
/// traps: never
/// example: `var a = [1, 2, 3, 4]; a.stable_partition_in_place(x => x % 2 == 0); a == [2, 4, 1, 3]`
pub fn stable_partition_in_place<S: IndexedMut, T>(mut self: S, keep: T -> Bool) -> Int:
    var yes: List<T> = []
    var no: List<T> = []
    for i in 0..self.len:
        let x = self.at(i)
        if keep(x): yes.push(x) else: no.push(x)
    let mid = yes.len
    for i in 0..mid: self[i] = yes[i]
    for i in 0..no.len: self[mid + i] = no[i]
    mid

// --- Differences and cut-indexed forms (decision 124) ---

/// O(n), O(n) memory. `[a[0], a[1] - a[0], ...]` (inverse of `scan(+)`).
///
/// cost: O(n), O(n) memory
/// empty: `[]`
/// order: a new `List`, keyed from 0
/// traps: never
/// example: `[3, 5, 4].adjacent_diff() == [3, 2, -1]`
pub fn adjacent_diff<S: Seq, T>(self: S) -> List<T>:
    var out: List<T> = []
    var prev: ?T = None
    for x in self:
        if let p = prev: out.push(x - p)
        else: out.push(x)
        prev = Some(x)
    out

/// O(1). Explicit piece `Range<D>` between cuts `lo` and `hi` (explicit twin of `lo..hi` in decision 124; traps if `lo > hi`).
///
/// cost: O(1)
/// traps: when `lo > hi`
/// example: `span(D.cuts.first!, D.cuts.last!) == D.all`
/// order: increasing, from `lo` to `hi`
pub fn span<D>(lo: Cut<D>, hi: Cut<D>) -> Range<D> = todo

// --- Sorted lookup, cut partition point, sorted set ops, and linear-time sorts ---

/// O(log n). Cut between the false prefix and true suffix of monotone `ok` on `self` (total; never `None`).
///
/// cost: O(log n) calls of `ok`
/// empty: the only cut
/// traps: never in release; debug traps when `ok` is not false-then-true at the answer
/// example: `[1, 3, 5].partition_point(x => x > 2).r == 1`
/// ties: none: a monotone `ok` has one flip
pub fn partition_point<D, T>(self: [D]T, ok: T -> Bool) -> Cut<D>:
    var (lo, hi) = (0, D.K)
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if ok(self[D.nth(m)]): hi = m else: lo = m + 1
    D.cuts.nth(lo)

/// O(log n). Contiguous range of elements in sorted `self` with `by(x) == target` (empty at insertion cut if absent).
///
/// cost: O(log n)
/// empty: an empty range
/// order: the keys of `self` holding `target`, increasing
/// traps: never; unsorted input gives an unspecified range
/// example: `[1, 3, 3, 5].equal_range(3) == 1..3`
pub fn equal_range<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> Range<D>:
    var (lo, hi) = (0, D.K)
    while lo < hi:
        let m = lo + (hi - lo) / 2
        if by(self[D.nth(m)]) < target: lo = m + 1 else: hi = m
    var (a, b) = (lo, D.K)
    while a < b:
        let m = a + (b - a) / 2
        if by(self[D.nth(m)]) > target: b = m else: a = m + 1
    D.cuts.nth(lo)..D.cuts.nth(a)

/// O(|a| + |b|), O(|a| + |b|) memory. Sorted deduplicated union of two sorted sequences.
///
/// cost: O(|a| + |b|), O(|a| + |b|) memory
/// empty: the other input, deduplicated
/// ties: one copy of each value
/// order: increasing; a new `List`, keyed from 0
/// traps: never
/// example: `sorted_union([1, 3], [2, 3]) == [1, 2, 3]`
pub fn sorted_union<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < a.len or j < b.len:
        let x = if j == b.len or (i < a.len and a.nth(i) < b.nth(j)): a.nth(i) else: b.nth(j)
        if out.len == 0 or out[out.len - 1] != x: out.push(x)
        if i < a.len and a.nth(i) == x: i += 1
        else: j += 1
    out

///
///
/// cost: O(|a| + |b|), O(|a| + |b|) memory
/// empty: `[]`
/// ties: each value kept min(count in `a`, count in `b`) times
/// order: increasing; a new `List`, keyed from 0
/// traps: never
/// example: `sorted_intersect([1, 2, 2, 3], [2, 2, 4]) == [2, 2]`
pub fn sorted_intersect<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < a.len and j < b.len:
        if a.nth(i) < b.nth(j): i += 1
        elif b.nth(j) < a.nth(i): j += 1
        else:
            out.push(a.nth(i))
            i += 1
            j += 1
    out

/// O(|a| + |b|), O(|a| + |b|) memory. Sorted elements in `a` not in `b`.
///
/// cost: O(|a| + |b|), O(|a| + |b|) memory
/// empty: `a`, copied, when `b` is empty
/// ties: each value kept (count in `a` − count in `b`) times, at least 0
/// order: increasing; a new `List`, keyed from 0
/// traps: never
/// example: `sorted_diff([1, 2, 2, 3], [2]) == [1, 2, 3]`
pub fn sorted_diff<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<T>:
    var out: List<T> = []
    var j = 0
    for i in 0..a.len:
        let x = a.nth(i)
        while j < b.len and b.nth(j) < x: j += 1
        if j < b.len and b.nth(j) == x: j += 1
        else: out.push(x)
    out

/// O(N log k), O(N) memory. Merge `k` sorted sequences via min-heap.
///
/// cost: O(N log k), O(N) memory
/// empty: `[]`
/// ties: the earlier list first (stable)
/// order: non-decreasing; a new `List`, keyed from 0
/// traps: never
/// example: `kway_merge([[1, 4], [2], [3]]) == [1, 2, 3, 4]`
pub fn kway_merge<S: Indexed, T>(lists: List<S>) -> List<T>:
    if lists.len == 0: return []
    var parts: List<List<T>> = [[x for x in l] for l in lists]
    while parts.len > 1:
        var next: List<List<T>> = []
        var i = 0
        while i + 1 < parts.len:
            next.push(merge_sorted(parts[i], parts[i + 1]))
            i += 2
        if i < parts.len: next.push(parts[i].copy())
        parts = next
    parts[0].copy()

/// O(n), O(n) memory. Stable LSD radix sort in place by integer key `by:` (signed `Int` supported).
///
/// cost: O(n) passes over 8-bit digits of the key, O(n) memory
/// empty: does nothing
/// ties: stable
/// traps: never
/// example: `var a = [3, -1, 2]; a.radix_sort(); a == [-1, 2, 3]`
pub fn radix_sort<S: IndexedMut, T>(mut self: S, by: T -> Int = x => x):
    let n = self.len
    if n <= 1: return
    var cur: List<(Int, T)> = []
    for i in 0..n: cur.push((by(self.at(i)), self.at(i)))
    var lo = cur[0].0
    var hi = cur[0].0
    for p in cur:
        if p.0 < lo: lo = p.0
        if hi < p.0: hi = p.0
    var span = hi - lo
    var shift = 0
    while span > 0:
        var count = [0 for _ in 0..257]
        for p in cur: count[((p.0 - lo) >> shift) % 256 + 1] += 1
        for d in 0..256: count[d + 1] += count[d]
        var out = cur.copy()
        for p in cur:
            let d = ((p.0 - lo) >> shift) % 256
            out[count[d]] = p
            count[d] += 1
        cur = out
        shift += 8
        span = span >> 8
    for i in 0..n: self[i] = cur[i].1

/// O(n + |K|), O(n + |K|) memory. Stable counting sort in place by domain or bounded-range key `by:`.
///
/// cost: O(n + |K|), O(n + |K|) memory
/// empty: does nothing
/// ties: stable
/// traps: never
/// example: `var a = ['b', 'a', 'b']; a.counting_sort(by: c => c); a == ['a', 'b', 'b']`
pub fn counting_sort<S: IndexedMut, T, K>(mut self: S, by: T -> K) = todo

/// O(n) expected, O(n) memory. Whether `a` and `b` have the same multiset of elements.
///
/// cost: O(n) expected, O(n) memory
/// empty: `true` when both are empty
/// traps: never
/// example: `is_permutation([1, 2, 2], [2, 1, 2]) == true`
pub fn is_permutation<A: Seq, B: Seq, T>(a: A, b: B) -> Bool:
    var left = HashMap<T, Int>()
    var n = 0
    for x in a:
        let c = left.get(x)
        left[x] = if c == None: 1 else: c! + 1
        n += 1
    for y in b:
        let c = left.get(y)
        if c == None or c! == 0: return false
        left[y] = c! - 1
        n -= 1
    n == 0

/// O(|self| + |of|). Whether `self` occurs as a subsequence of `of`.
///
/// cost: O(|self| + |of|)
/// empty: `true` for an empty `self`
/// traps: never
/// example: `"ace".is_subsequence("abcde") == true`
pub fn is_subsequence<S: Seq, N: Indexed, T>(self: S, of: N) -> Bool:
    var j = 0
    for x in self:
        while j < of.len and of.nth(j) != x: j += 1
        if j == of.len: return false
        j += 1
    true

// --- Permutations, combinatorics, subsequences, and monotone-stack trees ---

/// O(n log n), O(n) memory. Lexicographic rank of permutation `p` in `0..|V|!` (`|V| <= 20`).
///
/// cost: O(n log n), O(n) memory
/// empty: `0`
/// traps: when `|V| > 20`; debug traps when `p` is not a permutation
/// example: `perm_rank([1, 0, 2]) == 2`
pub fn perm_rank<V>(p: [V]V) -> Int:
    let n = V.K
    var bit: List<Int> = List.filled(n + 1, 0)
    var rank = 0
    var fact = 1
    for k in (0..n).rev:
        let v = p[V.nth(k)].ord
        var (j, smaller) = (v, 0)
        while j > 0:
            smaller += bit[j]
            j -= j & -j
        rank += smaller * fact
        fact *= n - k
        j = v + 1
        while j <= n:
            bit[j] += 1
            j += j & -j
    rank

/// O(n log n), O(n) memory. The `rank`-th lexicographic permutation of `V` (`0 <= rank < |V|!`).
///
/// cost: O(n log n), O(n) memory
/// empty: the empty permutation for `rank == 0`
/// order: keyed by `V`
/// traps: when `rank` is not in `0..|V|!`
/// example: `perm_unrank<[3]>(2) == [1, 0, 2]`
pub fn perm_unrank<V>(rank: Int) -> [V]V:
    let n = V.K
    var digits: List<Int> = List.filled(n, 0)
    var r = rank
    for k in 1..=n:
        digits[n - k] = r % k
        r /= k
    assert r == 0, "`rank` is at least |V|!"
    var bit: List<Int> = List.filled(n + 1, 0)
    for j in 1..=n:
        bit[j] += 1
        if j + (j & -j) <= n: bit[j + (j & -j)] += bit[j]
    var top = 1
    while top * 2 <= n: top *= 2
    var out: [V]V = V.nth(0)
    for k in 0..n:
        var (pos, need, step) = (0, digits[k] + 1, top)
        while step > 0:
            if pos + step <= n and bit[pos + step] < need:
                pos += step
                need -= bit[pos]
            step /= 2
        out[V.nth(k)] = V.nth(pos)
        var j = pos + 1
        while j <= n:
            bit[j] -= 1
            j += j & -j
    out

/// O(n), O(n) memory. Sign/parity of permutation `p` (`+1` even, `-1` odd).
///
/// cost: O(n), O(n) memory
/// empty: `1`
/// traps: never in release; debug traps when `p` is not a permutation
/// example: `perm_sign([1, 0, 2]) == -1`
pub fn perm_sign<V>(p: [V]V) -> Int:
    var seen: [V]Bool = false
    var sign = 1
    for v in V:
        if seen[v]: continue
        var (u, len) = (v, 0)
        while not seen[u]:
            seen[u] = true
            u = p[u]
            len += 1
        if len % 2 == 0: sign = -sign
    sign

/// O(n), O(n) memory. Order of permutation `p` (LCM of disjoint cycle lengths).
///
/// cost: O(n), O(n) memory
/// empty: `1`
/// traps: never; the order can overflow `Int` past about |V| = 400
/// example: `perm_order([1, 0, 3, 4, 2]) == 6`
pub fn perm_order<V>(p: [V]V) -> Int:
    var seen: [V]Bool = false
    var ord = 1
    for v in V:
        if seen[v]: continue
        var (u, len) = (v, 0)
        while not seen[u]:
            seen[u] = true
            u = p[u]
            len += 1
        var (a, b) = (ord, len)
        while b != 0: (a, b) = (b, a % b)
        ord = ord / a * len
    ord

/// O(|I| + |V|), O(|I| + |V|) memory. Inverse of partial injection `self: [I]?V -> [V]?I` (traps on duplicate non-`None` images).
///
/// cost: O(|I| + |V|), O(|I| + |V|) memory
/// empty: every entry `None`
/// order: keyed by `V`
/// traps: when two keys map to the same value
/// example: `[Some(2), None, Some(0)].partial_inverse() == [Some(2), None, Some(0)]`
pub fn partial_inverse<I, V>(self: [I]?V) -> [V]?I:
    var out: [V]?I = None
    for i in I:
        if let v = self[i]:
            assert out[v] == None, "two positions map to the same value"
            out[v] = i
    out


/// O(k) per combination. All `k`-element combinations in lexicographic index order (`k = 2` is unordered pairs).
///
/// cost: O(k) per combination
/// empty: one empty combination when `k == 0`, else nothing
/// order: lexicographic by position
/// traps: when `k < 0`
/// example: `[1, 2, 3].combinations(2).collect() == [[1, 2], [1, 3], [2, 3]]`
pub fn combinations<S: Indexed, T>(self: S, k: Int) -> yields List<T>:
    assert k >= 0
    let n = self.len
    if k > n: return
    var idx = [i for i in 0..k]
    while true:
        yield [self.nth(i) for i in idx]
        var i = k - 1
        while i >= 0 and idx[i] == n - k + i: i -= 1
        if i < 0: return
        idx[i] += 1
        for j in i + 1..k: idx[j] = idx[j - 1] + 1

/// O(1) per pair. Zero-alloc unordered index pairs `(i, j)` with `i < j` over `self: Range<D>`.
///
/// cost: O(1) per pair
/// empty: yields nothing
/// order: by the first key, then the second
/// traps: never
/// example: `(0..3).index_pairs().collect() == [(0, 1), (0, 2), (1, 2)]`
pub fn index_pairs<D>(self: Range<D>) -> yields (D, D):
    for i in 0..self.len:
        for j in i + 1..self.len: yield (self[i], self[j])

/// O(k) per permutation. All `k`-permutations of `self` (default `k = self.len`).
///
/// cost: O(k) per permutation
/// empty: one empty permutation
/// ties: equal elements give repeated permutations; dedup the input's arrangements with `next_permutation`
/// order: lexicographic by position
/// traps: when `k` is not in `0..=self.len`
/// example: `[1, 2].permutations().collect() == [[1, 2], [2, 1]]`
pub fn permutations<S: Indexed, T>(self: S, k: ?Int = None) -> yields List<T>:
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

/// O(1). Steerable item yielded by `subsequences` (decision 126, W6).
pub struct SubseqStep<T>(items: List<T>):
    /// O(1). Skip extending this prefix in the `subsequences` search tree.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2].subsequences().filter(s => s.items.len <= 1 or { s.prune(); false }).count() == 3`
    pub fn prune(mut self) = todo

/// O(k) per subsequence. All subsequences of `self` in lex order; supports `step.prune()` (decision 126, W6).
///
/// cost: O(k) per subsequence
/// empty: one empty subsequence
/// order: lexicographic by position, the empty one first; `prune` skips every extension of the current one
/// traps: never
/// example: `[1, 2].subsequences().map(s => s.items).collect() == [[], [1], [1, 2], [2]]`
pub fn subsequences<S: Indexed, T>(self: S) -> yields SubseqStep<T> = todo

/// O(n log n), O(n) memory. The positions of one longest increasing subsequence by `by`;
/// `strict: false` allows equal neighbours. Longest non-increasing is `by: desc`-style `x => -x`
/// with `strict: false`.
///
/// cost: O(n log n), O(n) memory
/// empty: `[]`
/// ties: among longest subsequences, the one whose positions are lexicographically greatest from the end (the patience-sorting choice)
/// order: increasing positions of `self`
/// traps: never
/// example: `[3, 1, 2, 2, 5].lis() == [1, 2, 4]`
/// example: `[3, 1, 2, 2, 5].lis(strict: false).len == 4`
pub fn lis<D, T, K>(self: [D]T, strict: Bool = true, by: T -> K = x => x) -> List<D>:
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
    var cur: ?D = if tails.len > 0: Some(tails[tails.len - 1]) else: None
    while let c = cur:
        out.push(c)
        cur = prev[c]
    var res: List<D> = []
    for i in (0..out.len).rev: res.push(out[i])
    res

/// O(|a| * |b|), O(|a| * |b|) memory. Longest common subsequence of `a` and `b` as matched `(i, j)` index pairs.
///
/// cost: O(|a| · |b|), O(|a| · |b|) memory
/// empty: `[]`
/// ties: among longest, the one the standard backward walk finds, preferring to step in `a` first
/// order: increasing `(i, j)`
/// traps: never
/// example: `lcs("abc", "acb").len == 2`
pub fn lcs<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<(Int, Int)>:
    let (n, m) = (a.len, b.len)
    var f = [[0 for _ in 0..m + 1] for _ in 0..n + 1]
    for i in 0..n:
        for j in 0..m:
            f[i + 1][j + 1] = if a.nth(i) == b.nth(j): f[i][j] + 1 elif f[i][j + 1] < f[i + 1][j]: f[i + 1][j] else: f[i][j + 1]
    var out: List<(Int, Int)> = []
    var (i, j) = (n, m)
    while i > 0 and j > 0:
        if a.nth(i - 1) == b.nth(j - 1):
            out.push((i - 1, j - 1))
            i -= 1
            j -= 1
        elif f[i - 1][j] == f[i][j]: i -= 1
        else: j -= 1
    reverse(mut out)
    out

/// O(n), O(n) memory. Min-heap (or max-heap when `min_root: false`) Cartesian tree in one monotone-stack pass.
///
/// cost: O(n), O(n) memory
/// empty: `root` is `None` and every array is empty
/// ties: the earlier position is the ancestor
/// order: arrays keyed by `D`
/// traps: never
/// example: `[3, 1, 2].cartesian_tree().root == Some(1)`
pub fn cartesian_tree<D, T>(self: [D]T, min_root: Bool = true) -> (root: ?D, left: [D]?D, right: [D]?D, parent: [D]?D):
    var left: [D]?D = None
    var right: [D]?D = None
    var parent: [D]?D = None
    var st: List<D> = []
    for i in D:
        var last: ?D = None
        while st.len > 0 and (if min_root: self[st[st.len - 1]] > self[i] else: self[st[st.len - 1]] < self[i]):
            last = st.pop()
        if let l = last:
            left[i] = l
            parent[l] = i
        if st.len > 0:
            let t = st[st.len - 1]
            right[t] = i
            parent[i] = t
        st.push(i)
    (root: if st.len > 0: Some(st[0]) else: None, left: left, right: right, parent: parent)

// --- Incremental / online sequence structures (with mark/undo_to rollback for tree DFS) ---

/// O(1) construction; O(log n) push, O(1) amortized rollback. Online patience-sorting LIS tracker.
pub struct OnlineLis<T>(strict: Bool = true):
    /// O(log n). Append `x` and return the length of the longest increasing subsequence ending at `x`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); t.push(5); t.push(1); t.push(7) == 2`
    pub fn push(mut self, x: T) -> Int = todo
    /// O(1). Current global LIS length.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); t.push(1); t.push(3); t.len == 2`
    @get pub fn len(self) -> Int = todo
    /// O(1). Whether no elements have been pushed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `OnlineLis<Int>().is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo
    /// O(1). Reset all state.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); t.push(1); t.clear(); t.len == 0`
    pub fn clear(mut self) = todo
    /// O(n), O(n) memory. Reconstruct one global LIS as 0-based push indices.
    ///
    /// cost: O(n), O(n) memory
    /// empty: `[]`
    /// ties: the subsequence ending at the latest push of maximal length
    /// order: increasing push indices
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); for x in [3, 1, 2]: t.push(x); t.reconstruct() == [1, 2]`
    pub fn reconstruct(self) -> List<Int> = todo
    /// O(1). Save a rollback checkpoint for DFS-on-tree LIS.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = OnlineLis<Int>(); let m = t.mark(); t.push(1); t.undo_to(m); t.len == 0`
    pub fn mark(self) -> Int = todo
    /// O(1) amortized. Roll back all `push`es since checkpoint `m`.
    ///
    /// cost: O(1) amortized per undone push
    /// traps: when `m` is not a live mark of this tracker
    /// example: `var t = OnlineLis<Int>(); t.push(2); let m = t.mark(); t.push(5); t.undo_to(m); t.len == 1`
    pub fn undo_to(mut self, m: Int) = todo

/// O(1) construction; O(1) amortized push and rollback. Online monotone stack returning nearest dominating index.
pub struct MonoStack<D, T>(keep_smaller: Bool = true, strict: Bool = true):
    /// O(1) amortized. Push `(i, x)`, pop dominated entries, and return the nearest dominating index (or `None`).
    ///
    /// cost: O(1) amortized
    /// ties: equal values pop unless `strict: false`
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); s.push(1, 3) == Some(0)`
    pub fn push(mut self, i: D, x: T) -> ?D = todo
    /// O(1). Current top `(index, val)`, or `None` when empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(3, 8); s.top == Some((index: 3, val: 8))`
    /// ties: none: the stack has one top
    @get pub fn top(self) -> ?(index: D, val: T) = todo
    /// O(1). Current stack depth.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); s.len == 1`
    @get pub fn len(self) -> Int = todo
    /// O(1). Whether the stack is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MonoStack<Int, Int>().is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo
    /// O(1). Empty the stack and rollback log.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 1); s.clear(); s.is_empty`
    pub fn clear(mut self) = todo
    /// O(1). Save a rollback checkpoint for DFS-on-tree monotone stacks.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = MonoStack<Int, Int>(); let m = s.mark(); s.push(0, 1); s.undo_to(m); s.is_empty`
    pub fn mark(self) -> Int = todo
    /// O(1) amortized. Roll back all `push`es since checkpoint `m`.
    ///
    /// cost: O(1) amortized per undone push
    /// traps: when `m` is not a live mark of this stack
    /// example: `var s = MonoStack<Int, Int>(); s.push(0, 5); let m = s.mark(); s.push(1, 9); s.undo_to(m); s.len == 1`
    pub fn undo_to(mut self, m: Int) = todo

@test
fn scan_folds_every_prefix() = todo

@test
fn sorted_lookup_and_linear_search():
    // Sequence first_ge / last_le return indices (?Int on List, ?D on [D]T)
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_ge(3) == 1 and xs.first_ge(4) == 3 and xs.first_ge(11) == None
    assert xs.last_le(3) == 2 and xs.last_le(0) == None and xs.last_le(99) == 4
    assert xs.find(7) == 3 and xs.find(8) == None
    domain V = 1..=5
    let a = [V] v => xs[v.to_idx]
    assert a.first_ge(4) == V(4) and a.last_le(3) == V(3) and a.find(10) == V(5)
    assert first(x for x in xs if x > 3) == 7 and first(x for x in xs if x > 10) == None
    let words = ["pear", "fig", "banana", "kiwi"]
    assert words.min(by: w => w.len) == "fig" and words.max(by: w => w.len) == "banana"
    let nothing: List<Str> = []
    assert nothing.min(by: w => w.len) == None
    domain D = 0..5
    assert [w.len for w in D.windows(3)] == [3, 3, 3] and D.windows(3)[1] == (D(1)..=D(3))

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
    var fwd = [p.copy()]
    while next_permutation(mut p): fwd.push(p.copy())
    assert fwd == [[1, 2, 2], [2, 1, 2], [2, 2, 1]] and p == [1, 2, 2]
    var q = [2, 2, 1]
    var bwd = [q.copy()]
    while prev_permutation(mut q): bwd.push(q.copy())
    assert bwd == [[2, 2, 1], [2, 1, 2], [1, 2, 2]] and q == [2, 2, 1]
    // 4-element full round-trip forward and backward
    var perm = [1, 2, 3, 4]
    var up = [perm.copy()]
    while next_permutation(mut perm): up.push(perm.copy())
    assert up.len == 24 and perm == [1, 2, 3, 4]
    var perm_down = [4, 3, 2, 1]
    var down = [perm_down.copy()]
    while prev_permutation(mut perm_down): down.push(perm_down.copy())
    assert down.len == 24 and perm_down == [4, 3, 2, 1]
    assert down == up.rev
    // Edge cases: empty and singleton
    var empty: List<Int> = []
    assert not next_permutation(mut empty) and not prev_permutation(mut empty)
    var one = [42]
    assert not next_permutation(mut one) and not prev_permutation(mut one) and one == [42]
    assert runs("aabccc") == [('a', 2), ('b', 1), ('c', 3)]

@test
fn compress_gives_the_values_their_own_domain():
    let ys = [40, -7, 40, 1_000_000_000, 3]
    let xs = compress(ys)
    assert xs.len == 4 and xs.D.K == 4
    assert [xs[i] for i in xs.D] == [-7, 3, 40, 1_000_000_000]
    var cnt: [xs.D]Int = 0
    for y in ys: cnt[xs.at(y)] += 1
    assert [cnt[i] for i in xs.D] == [1, 1, 2, 1]
    assert xs.first_ge(4) == xs.at(40) and xs.first_ge(2_000_000_000) == None
    assert xs.last_le(4) == xs.at(3) and xs.last_le(-100) == None
    let other = compress([5, 6])
    assert other.D.K == 2 and xs.D.K == 4
    // Compressed range slicing
    let blk = xs.D.first..=xs.D.nth(2)
    assert xs.slice(blk) == [-7, 3, 40]
    assert xs[blk] == [-7, 3, 40]
    assert xs.slice(xs.D.nth(1)..xs.D.nth(1)) == []

@test
fn min_max_by_tie_breaking() = todo

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
fn list_insert_remove_take_and_swap_at() = todo

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
            if k not in ks: ks.push(k)
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
fn deduped_and_partition_leave_the_list_alone() = todo

@test
fn get_or_insert_stores_only_what_is_missing():
    var ids = Map<Str, Int>()
    assert ids.get_or_insert("a", ids.len) == 0 and ids.get_or_insert("b", ids.len) == 1
    assert ids.get_or_insert("a", 99) == 0 and ids.len == 2

@test
fn get_or_insert_with_makes_a_value_only_when_missing():
    var ids = Map<Str, Int>()
    var made = 0
    fn fresh() -> Int:
        made += 1
        made * 10
    assert ids.get_or_insert_with("a", fresh) == 10 and ids.get_or_insert_with("a", fresh) == 10
    assert ids.get_or_insert_with("b", () => 7) == 7 and made == 1

@test
fn update_starts_from_init_and_then_from_the_value() = todo

@test
fn runs_with_by_groups_neighbours_with_the_same_key():
    let rs = ["ab", "ac", "b", "ad"].runs(by: s => s[0])
    assert rs == [('a', 2), ('b', 1), ('a', 1)]
    let none_yet: List<Int> = []
    assert none_yet.runs(by: x => x).len == 0

@test
fn excl_fold_folds_all_but_one():
    assert [(x, rest) for (x, rest) in [2, 3, 5].excl_fold(*)] == [(2, 15), (3, 10), (5, 6)]
    assert [
        rest for (_, rest) in ["a", "b", "c"].excl_fold(Monoid("", (p, q) => p + q))
    ] == ["bc", "ac", "ab"]
    assert [rest for (_, rest) in [7].excl_fold(+)] == [0]

@test
fn windows_and_zip_walk_in_step():
    assert [w for w in [1, 2, 3, 4].windows(2)] == [[1, 2], [2, 3], [3, 4]]
    assert [w for w in [1, 2].windows(2)] == [[1, 2]] and [w for w in [1, 2].windows(3)].len == 0
    assert zip([1, 2, 3], ["a", "b"]) == [(1, "a"), (2, "b")] and zip([1], List<Int>()).len == 0

@test
fn pairs_and_with_prev_yield_neighbours():
    assert [p for p in [10, 20, 30].pairs()] == [(10, 20), (20, 30)]
    assert [p for p in [7].pairs()].len == 0
    let empty: List<Int> = []
    assert [p for p in empty.pairs()].len == 0
    assert [(p ?? -1, x) for (p, x) in [10, 20, 30].with_prev()] == [(-1, 10), (10, 20), (20, 30)]
    assert [(p ?? -1, x) for (p, x) in [7].with_prev()] == [(-1, 7)]
    assert [p for p in empty.with_prev()].len == 0

@test
fn cyclic_pairs_close_the_ring():
    assert [p for p in [1, 2, 3].cyclic_pairs()] == [(1, 2), (2, 3), (3, 1)]
    assert [p for p in [7].cyclic_pairs()] == [(7, 7)]
    let empty: List<Int> = []
    assert [p for p in empty.cyclic_pairs()].len == 0

/// The index of the first largest value (optionally under key `by:`), or `None` when empty:
/// `[3, 9, 2, 9].argmax()` is `Some(1)` and `words.argmax(by: w => w.len)` is the index of the
/// longest word. It is an index of `self`, so `a[a.argmax()!]` is the largest value of a
/// `List<T>`, a `[1..=n]T`, a `[V]T` or a filtered view of one alike. O(n).
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in key order
/// traps: never
/// example: `[3, 9, 2, 9].argmax() == Some(1)`
pub fn argmax<S: Keyed, D, T, K>(self: S, by: T -> K = x => x) -> ?D:
    var best: ?D = None
    var high: ?K = None
    for (i, x) in self.items:
        let k = by(x)
        if high == None or high! < k:
            best = Some(i)
            high = Some(k)
    best

/// The index of the first smallest value (optionally under key `by:`), or `None` when empty.
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in key order
/// traps: never
/// example: `[3, 1, 1].argmin() == Some(1)`
pub fn argmin<S: Keyed, D, T, K>(self: S, by: T -> K = x => x) -> ?D:
    var best: ?D = None
    var low: ?K = None
    for (i, x) in self.items:
        let k = by(x)
        if low == None or k < low!:
            best = Some(i)
            low = Some(k)
    best

@test
fn argmax_and_argmin_give_the_first_best_index() = todo

@test
fn prod_and_is_sorted_cover_sequences() = todo

@test
fn select_merge_sorted_and_rotate_right() = todo

@test
fn chunk_by_run_ranges_buckets_and_chunks() = todo

@test
fn suffix_scan_matches_right_to_left_fold() = todo

@test
fn first_gt_and_last_lt_match_linear_scan():
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_gt(3) == Some(3) and xs.first_gt(10) == None and xs.first_gt(0) == Some(0)
    assert xs.last_lt(3) == Some(0) and xs.last_lt(1) == None and xs.last_lt(99) == Some(4)
    let c = compress([10, 3, 7, 3, 1])
    assert c.contains(7) and not c.contains(5)
    assert c.try_at(7) == c.at(7) and c.try_at(5) == None
    assert c.first_gt(3) == c.at(7) and c.first_gt(10) == None
    assert c.last_lt(3) == c.at(1) and c.last_lt(1) == None
    // Keyed lookup and domain array lookup
    let pairs = [(1, "a"), (3, "b"), (3, "c"), (7, "d")]
    assert pairs.first_gt(3, by: p => p.0) == Some(3)
    assert pairs.last_lt(3, by: p => p.0) == Some(0)
    domain V = 1..=5
    let va = [V] i => xs[i.to_idx]
    assert va.first_gt(3) == V(4) and va.last_lt(3) == V(1)
    assert xs[1..4].first_gt(3) == Some(2) and xs[1..4].last_lt(3) == None
    assert xs[0..4].last_lt(3) == Some(0) and xs[0..4].first_gt(7) == None
    assert pairs[1..4].first_gt(3, by: p => p.0) == Some(2)
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
    let p = [I] i => I([3, 5, 4, 1, 2][i.to_idx])
    let inv = p.inverse()
    for i in I:
        assert inv[p[i]] == i and p[inv[i]] == i
    domain P = 0..5
    let q = [P] i => I([3, 5, 4, 1, 2][i.to_idx])
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
        var brute = 0
        for i in 0..a.len:
            for j in i + 1..a.len:
                if a[i] > a[j]: brute += 1
        assert inversions(a) == brute

@test
fn monotone_stack_and_sliding_extrema() = todo

@test
fn seq_and_indexed_traits_unify_sequences_and_views() = todo

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
fn scan_prod_and_rotate_method_syntax() = todo

/// O(1). The pair the other way round: `(a, b).swap` is `(b, a)`, and labels stay in place,
/// so an edge `(u: a, v: b).swap` is `(u: b, v: a)`.
///
/// cost: O(1)
/// traps: never
/// example: `(u: 1, v: 2).swap == (u: 2, v: 1)`
@get pub fn swap<A>(self: (A, A)) -> (A, A) = todo

/// Maximal pieces where every adjacent pair links: `a.runs(link: (x, y) => y == x + 1)`.
///
/// cost: O(n)
/// empty: yields nothing
/// order: left to right; each run keeps the keys of `self`
/// traps: never
/// example: `[r.len for r in [1, 2, 4, 5, 6].runs_linked((x, y) => y == x + 1)] == [2, 3]`
pub fn runs_linked<S: Indexed, T>(self: S, link: (T, T) -> Bool) -> yields Slice<S> = todo

/// Pieces between elements where `sep` holds; separators excluded, empty pieces kept.
///
/// cost: O(n)
/// empty: yields one empty piece
/// order: left to right; each piece keeps the keys of `self`
/// traps: never
/// example: `[p.len for p in [1, 0, 0, 2].split(x => x == 0)] == [1, 0, 1]`
pub fn split<S: Indexed, T>(self: S, sep: T -> Bool) -> yields Slice<S> = todo

/// Which end of a piece a two-pointer result is keyed by.
pub enum Anchor = Start | End
/// Two pointers: for each right end, the longest piece ending there that is `ok`, fed through
/// `state: Track`. `ok` must be monotone under shrinking. `anchor: Anchor.Start` keys by left
/// end instead: the longest piece starting at each position, as greedy jumps want. O(n) amortized.
///
/// cost: O(n) amortized pushes and pops on `state`
/// empty: an empty array
/// ties: none: each key has one longest piece
/// order: keyed by `D`, by right end (or left end with `Anchor.Start`)
/// traps: never in release; debug traps when `ok` fails on a single element after a shrink
/// example: `[1, 1, 2].maximal(state: Counter<Int, Int>(0), ok: c => c.len <= 1).map(w => w.len) == [1, 2, 1]`
pub fn maximal<D, T, St>(self: [D]T, state: St, ok: St -> Bool, anchor: Anchor = Anchor.End) -> [D]Slice<[D]T> = todo

/// For each right end, the shortest piece ending there that is `ok`, or None, keyed by that
/// end like the ring forms; `anchor: Anchor.Start` keys by left end. O(n) amortized.
///
/// cost: O(n) amortized pushes and pops on `state`
/// empty: an empty array
/// ties: none: each key has one shortest piece
/// order: keyed by `D`, by right end (or left end with `Anchor.Start`)
/// traps: never
/// example: `[1, 2, 3].minimal(state: Window(+), ok: w => (w.all ?? 0) >= 3).map(w => w.map(.len)) == [None, Some(2), Some(1)]`
pub fn minimal<D, T, St>(self: [D]T, state: St, ok: St -> Bool, anchor: Anchor = Anchor.End) -> [D]?Slice<[D]T> = todo

/// O(1) to make. `a` read as a cycle: windows and two pointers wrap around the end, and every
/// element is still a position of `a`. Replaces doubling the array.
///
/// cost: O(1)
/// empty: a ring with no positions
/// traps: never
/// example: `[1, 2, 3].ring.next(2) == 0`
@get pub fn ring<D, T>(self: [D]T) -> Ring<D, T> = Ring(self)

/// `a.ring`; see `ring`.
pub struct Ring<D, T>(src: [D]T):
    /// The position `d` steps after `i` (before it when `d < 0`), wrapping: a forced-online
    /// decode `(v + last - 1) % n + 1` is `V.ring.step(v, last)`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.step(0, -1) == 2`
    pub fn step(self, i: D, d: Int) -> D = D.nth(((i.ord + d) % D.K + D.K) % D.K)

    /// The position after `i`, wrapping.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.next(2) == 0`
    pub fn next(self, i: D) -> D = self.step(i, 1)

    /// The position before `i`, wrapping.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.prev(0) == 2`
    pub fn prev(self, i: D) -> D = self.step(i, -1)

    /// All n positions, starting at `i` and wrapping. O(n) to walk.
    ///
    /// cost: O(1) per step
    /// empty: yields nothing on an empty ring
    /// order: `i`, then each next position, n in all
    /// traps: never
    /// example: `[1, 2, 3].ring.from(1).collect() == [1, 2, 0]`
    pub fn from(self, i: D) -> yields D:
        for k in 0..D.K: yield self.step(i, k)

    /// The `len` positions from `start` on, wrapping (`len <= n`).
    ///
    /// cost: O(1)
    /// empty: an empty span when `len == 0`
    /// traps: when `len` is not in `0..=n`
    /// example: `[1, 2, 3].ring.span(2, 2).end == 0`
    pub fn span(self, start: D, len: Int) -> RingRange<D> = todo

    /// The n windows of `k` elements, one starting at each position, wrapping. O(n) to walk.
    ///
    /// cost: O(1) per window
    /// empty: yields nothing
    /// order: by start position, increasing
    /// traps: when `k` is not in `1..=n`
    /// example: `[1, 2, 3].ring.windows(2).count() == 3`
    pub fn windows(self, k: Int) -> yields RingSlice<D, T> = todo

    /// Two pointers round the cycle: for each position as the last element, the longest
    /// wrapping piece (at most n elements) ending there that is `ok`, fed through
    /// `state: Track`. O(n) pushes and pops (two laps).
    ///
    /// cost: O(n) pushes and pops on `state`
    /// empty: an empty array
    /// order: keyed by `D`, by last element
    /// traps: never
    /// example: `[1, 1, 2].ring.maximal(state: Counter<Int, Int>(0), ok: c => c.len <= 2).map(w => w.len) == [3, 3, 3]`
    /// ties: none: each last element has one longest piece
    pub fn maximal<St>(self, state: St, ok: St -> Bool) -> [D]RingSlice<D, T> = todo

    /// For each position as the last element, the shortest wrapping piece ending there that is
    /// `ok`, or None. O(n) pushes and pops (two laps).
    ///
    /// cost: O(n) pushes and pops on `state`
    /// empty: an empty array
    /// order: keyed by `D`, by last element
    /// traps: never
    /// example: `[2, 0, 1].ring.minimal(state: Window(+), ok: w => (w.all ?? 0) >= 3).map(w => w.map(.len)) == [Some(2), Some(3), Some(3)]`
    /// ties: none: each last element has one shortest piece
    pub fn minimal<St>(self, state: St, ok: St -> Bool) -> [D]?RingSlice<D, T> = todo

/// A run of positions of `D` read cyclically, possibly wrapping past the last position, at most
/// `D.count` of them. The keys-only half of a `RingSlice`, so it needs no array.
pub struct RingRange<D>():
    ///
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).len == 2`
    @get pub fn len(self) -> Int = todo

    /// O(1). The first position.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).start == 2`
    @get pub fn start(self) -> D = todo

    /// O(1). The last position.
    ///
    /// cost: O(1)
    /// traps: on an empty span
    /// example: `[1, 2, 3].ring.span(2, 2).end == 0`
    @get pub fn end(self) -> D = todo

    /// O(1) to make. Its positions in ring order.
    ///
    /// cost: O(1) to make, O(1) per step
    /// order: ring order from `start`
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).keys.collect() == [2, 0]`
    @get pub fn keys(self) -> yields D = todo

    ///
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).contains(0) == true`
    pub fn contains(self, i: D) -> Bool = todo

/// A contiguous piece of a ring, possibly wrapping past the end, at most n elements.
pub struct RingSlice<D, T>():
    /// O(1). Its positions, without the array.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.range.len == 2`
    @get pub fn range(self) -> RingRange<D> = todo

    ///
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[w.len for w in [1, 2, 3].ring.windows(2)] == [2, 2, 2]`
    @get pub fn len(self) -> Int = todo

    /// O(1). The first position.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.start == 0`
    @get pub fn start(self) -> D = todo

    /// O(1). The last position.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.end == 1`
    @get pub fn end(self) -> D = todo

    /// O(1) to make. Its positions in ring order.
    ///
    /// cost: O(1) to make, O(1) per step
    /// order: ring order from `start`
    /// traps: never
    /// example: `[1, 2, 3].ring.windows(2).first!.keys.collect() == [0, 1]`
    @get pub fn keys(self) -> yields D = todo

/// The non-empty ranges whose every element is strictly below both elements just outside it
/// (both must exist). At most 2n of them, left ends ascending, from one monotone-stack pass;
/// the "walls" shape of grid problems such as IOI 2019 Rectangles. O(n) to walk.
///
/// cost: O(n) to walk
/// empty: yields nothing
/// order: left ends ascending, then shorter first
/// traps: never
/// example: `[3, 1, 2, 5].valleys().collect() == [1..2, 1..3, 2..3]`
pub fn valleys<D, T>(self: [D]T) -> yields Range<D> = todo

/// For each position, the widest piece where it is the minimum of `by` (ties: the left one
/// extends). O(n).
///
/// cost: O(n)
/// empty: an empty array
/// ties: the left of two equal minima extends over the right; the right stops at the left
/// order: keyed by `D`
/// traps: never
/// example: `[2, 1, 1].spans() == [0..1, 0..3, 2..3]`
pub fn spans<D, T, K>(self: [D]T, by: T -> K = x => x) -> [D]Range<D>:
    let left = self.prev_smaller(by: by, strict: false)
    let right = self.next_smaller(by: by, strict: true)
    [D] i => (if let l = left[i]: l.r else: D.cuts.nth(0))..(if let r = right[i]: r.l else: D.cuts.nth(D.K))

/// Nearest earlier position with a smaller key (`strict: false`: smaller or equal).
///
/// cost: O(n)
/// empty: an empty array
/// ties: equal keys do not count unless `strict: false`
/// order: keyed by `D`
/// traps: never
/// example: `[2, 1, 3].prev_smaller() == [None, None, Some(1)]`
pub fn prev_smaller<D, T, K>(self: [D]T, by: T -> K = x => x, strict: Bool = true) -> [D]?D:
    var out: [D]?D = None
    var st: List<D> = []
    for i in D:
        while st.len > 0 and (if strict: by(self[st[st.len - 1]]) >= by(self[i]) else: by(self[st[st.len - 1]]) > by(self[i])):
            let _ = st.pop()
        if st.len > 0: out[i] = st[st.len - 1]
        st.push(i)
    out

/// Nearest later position with a smaller key.
///
/// cost: O(n)
/// empty: an empty array
/// ties: equal keys do not count unless `strict: false`
/// order: keyed by `D`
/// traps: never
/// example: `[2, 1, 3].next_smaller() == [Some(1), None, None]`
pub fn next_smaller<D, T, K>(self: [D]T, by: T -> K = x => x, strict: Bool = true) -> [D]?D:
    var out: [D]?D = None
    var st: List<D> = []
    for k in (0..D.K).rev:
        let i = D.nth(k)
        while st.len > 0 and (if strict: by(self[st[st.len - 1]]) >= by(self[i]) else: by(self[st[st.len - 1]]) > by(self[i])):
            let _ = st.pop()
        if st.len > 0: out[i] = st[st.len - 1]
        st.push(i)
    out

/// Nearest earlier position with an equal key.
///
/// cost: O(n) expected
/// empty: an empty array
/// order: keyed by `D`
/// traps: never
/// example: `[1, 2, 1].prev_same() == [None, None, Some(0)]`
pub fn prev_same<D, T, K>(self: [D]T, by: T -> K = x => x) -> [D]?D:
    var out: [D]?D = None
    var seen = HashMap<K, D>()
    for i in D:
        let k = by(self[i])
        out[i] = seen.get(k)
        seen[k] = i
    out

/// Nearest later position with an equal key.
///
/// cost: O(n) expected
/// empty: an empty array
/// order: keyed by `D`
/// traps: never
/// example: `[1, 2, 1].next_same() == [Some(2), None, None]`
pub fn next_same<D, T, K>(self: [D]T, by: T -> K = x => x) -> [D]?D:
    var out: [D]?D = None
    var seen = HashMap<K, D>()
    for j in (0..D.K).rev:
        let i = D.nth(j)
        let k = by(self[i])
        out[i] = seen.get(k)
        seen[k] = i
    out

/// The ranges no other range contains, sorted by left end (ties: the first kept). Equal ranges
/// keep one. The usual first step of interval DP over covers. O(n log n).
///
/// cost: O(n log n)
/// empty: `[]`
/// ties: equal ranges keep the first
/// order: by left end; a new `List`, keyed from 0
/// traps: never
/// example: `[0..5, 1..3, 4..8].outermost() == [0..5, 4..8]`
pub fn outermost<D, S: Seq<Range<D>>>(self: S) -> List<Range<D>>:
    var rs: List<(Int, Range<D>)> = []
    for r in self: rs.push((rs.len, r))
    rs.sort(by: p => (p.1.start, -Int(p.1.end), p.0))
    var out: List<Range<D>> = []
    for (_, r) in rs:
        if out.len == 0 or out[out.len - 1].end < r.end: out.push(r)
    out

/// O(1). A list's keys: `0..xs.len`, in increasing order.
///
/// cost: O(1)
/// empty: `0..0`
/// order: increasing
/// traps: never
/// example: `[5, 6, 7].keys == 0..3`
@get pub fn keys<T>(self: List<T>) -> Range<Int> = 0..self.len

@test
fn dedup_first_some_tally_and_group():
    var a = [1, 1, 2, 1, 1]
    a.dedup()
    assert a == [1, 2, 1]
    var e: List<Int> = []
    e.dedup()
    assert e == []
    assert [1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None) == Some(40)
    assert [1, 2].first_some(x => if x > 3: Some(x) else: None) == None
    let t = [3, 1, 3, 3].tally()
    assert t[3] == 3 and t[1] == 1 and t.get(2) == None
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
