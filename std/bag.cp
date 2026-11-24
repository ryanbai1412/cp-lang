//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`.
//!
//! start: `SortedBag`, `SortedSet`, `SortedMap`, `BucketCount`
//! example: `var s: SortedSet<Int> = SortedSet(); s.add(5); s.add(2); s.add(9); s.nth(1) == 5 and s.rank(9) == 2`

use std.math.{ceil_sqrt}
use std.seq.{Seq, Track}

/// Read-only handle to one key's slot in a `SortedBag<K>`, valid until the bag's next `add`,
/// `remove` or `clear`, because a B+tree change may move keys between leaves (decision 579).
/// Using it after one traps on the interpreter and in `--checked` builds, naming the change;
/// on the judge the check is compiled out. Make cursors after the last change.
pub struct BagCursor(id: Int, stamp: Int)

const BAG_B = 32
const BAG_ADD = 1
const BAG_REMOVE = 2
const BAG_CLEAR = 3
const BAG_OPS = ["none", "add", "remove", "clear"]

/// An ordered multiset (bag) of `K` backed by an order-statistic B+tree over flat arrays: leaves of
/// up to 32 keys with their multiplicities, linked for \(O(1)\) cursor steps, under inner nodes
/// that keep each child's element count, with recycled node slots (decision 579).
/// `add`, `add_many`, `remove`, `remove_all`, `contains`, `count`,
/// `pop_first`, `pop_last`, `rank`, `nth`, `first_true`, `last_true`, and `cursor_*` are \(O(\log n)\);
/// `range(r)` returns an \(O(1)\) view whose `.first`, `.last`, `.len`, `.is_empty` are \(O(\log n)\)
/// expected and whose iteration (`for x in b.range(r)` / `.rev`) is \(O(\log n + k)\);
/// `first`, `last`, `key_of` and `count_of` are \(O(1)\); `next_cursor` and `prev_cursor` are \(O(1)\) amortized;
/// `len`, `distinct`, `is_empty`, `clear`, and `take` are \(O(1)\);
/// `entries`, `entries_rev`, and `to_list` are \(O(n)\).
///
/// Elements are ordered by `K`'s `<`; copies of one key are equal and indistinguishable, so
/// no method here has a tie to break between them. `n` in every cost is `len`.
pub struct SortedBag<K>:
    var keys: List<K>
    var ws: List<Int>
    // Per node: fill, next leaf, previous leaf, and its block in `kids` (or -1).
    var meta: List<Int>
    var kids: List<Int>
    var root: Int
    var height: Int
    var live: Int
    var total: Int
    var stamp: Int
    var last_op: Int
    var head: Int
    var tail: Int
    var free: Int
    var pay: Bool
    var got: Int

    /// Create an empty `SortedBag<K>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag(); b.len == 0`
    pub fn new() -> SortedBag<K> = SortedBag(
        [],
        [],
        [],
        [],
        -1,
        0,
        0,
        0,
        0,
        0,
        -1,
        -1,
        -1,
        false,
        -1,
    )

    /// Build a `SortedBag` containing every element of `xs`.
    ///
    /// Every copy counts: `SortedBag.of([2, 2])` holds two elements.
    ///
    /// cost: \(O(n \log n)\) expected
    /// empty: `SortedBag<Int>.of([]).len == 0`
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).to_list() == [1, 3, 3]`
    pub fn of(xs: List<K>) -> SortedBag<K>:
        var b = SortedBag<K>.new()
        if xs.len == 0: return b
        if xs.len > BAG_B:
            b.keys.reserve(BAG_B)
            b.ws.reserve(BAG_B)
            for x in xs: b.add(x)
            return b
        // Few enough for one leaf: sort the keys in place and count each run, with no
        // descent per element.
        b.keys.reserve(xs.len)
        b.ws.reserve(xs.len)
        for x in xs: b.keys.push(x)
        b.keys.sort()
        var d = 0
        for i in 0..xs.len:
            if d > 0 and not b.keys[d - 1] < b.keys[i]: b.ws[d - 1] += 1
            else:
                b.keys[d] = b.keys[i]
                b.ws.push(1)
                d += 1
        while b.keys.len > d: let _ = b.keys.pop()
        b.meta.reserve(4)
        for v in [d, -1, -1, -1]: b.meta.push(v)
        b.root = 0
        b.head = 0
        b.tail = 0
        b.live = d
        b.total = xs.len
        b.changed(BAG_ADD)
        b

    /// Total number of elements in the bag (counting duplicates).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).len == 3`
    @get pub fn len(self) -> Int = total

    /// Number of distinct keys currently in the bag.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).distinct == 2`
    @get pub fn distinct(self) -> Int:
        live

    /// Whether the bag has no elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([1]).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        root < 0

    /// Remove all elements from the bag.
    ///
    /// The node array keeps its capacity.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); b.clear(); b.len == 0 and b.first == None`
    pub fn clear(mut self):
        keys.clear()
        ws.clear()
        meta.clear()
        kids.clear()
        free = -1
        root = -1
        height = 0
        live = 0
        total = 0
        head = -1
        tail = -1
        self.changed(BAG_CLEAR)

    /// Move the bag's contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); let t = b.take(); b.is_empty and t.to_list() == [1, 2]`
    pub fn take(mut self) -> SortedBag<K>:
        var out = SortedBag<K>.new()
        self.swap(mut out)
        self.changed(BAG_CLEAR)
        out

    /// Insert one copy of `x` into the bag. Always returns `true`.
    ///
    /// The `Bool` matches `SortedSet.add`, so code can switch between the two.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag(); b.add(3) and b.add(3) and b.count(3) == 2`
    pub fn add(mut self, x: K) -> Bool = self.insert(x, 1, false)

    fn add_unique(mut self, x: K) -> Bool = self.insert(x, 1, true)

    /// Insert `c >= 0` copies of `x` into the bag.
    ///
    /// `c == 0` does nothing.
    ///
    /// cost: \(O(\log n)\) expected, independent of `c`
    /// traps: when `c < 0`
    /// example: `var b = SortedBag.of([1]); b.add_many(5, 3); b.len == 4 and b.count(5) == 3`
    pub fn add_many(mut self, x: K, c: 0..):
        assert c >= 0
        if c == 0: return
        let _ = self.insert(x, c, false)

    /// Remove one copy of `x` if present; returns `true` if a copy was removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2]); b.remove(2) and b.count(2) == 1 and not b.remove(7)`
    pub fn remove(mut self, x: K) -> Bool = self.erase(x, false) > 0

    /// Remove all copies of `x` and return how many were removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2, 3]); b.remove_all(2) == 2 and b.to_list() == [3]`
    /// example: `var b = SortedBag.of([3]); b.remove_all(2) == 0`
    pub fn remove_all(mut self, x: K) -> Int = self.erase(x, true)

    /// Remove one copy of the key at cursor `c`, as `remove(key_of(c))` would.
    ///
    /// `c` must come from this bag and no mutation may have happened since it was made.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never in release (a stale cursor removes an unspecified key); debug traps when the node of `c` has been freed
    /// example: `var b = SortedBag.of([2, 5, 5]); b.remove_at(b.cursor_ge(3)!); b.to_list() == [2, 5]`
    pub fn remove_at(mut self, c: BagCursor):
        self.check(c)
        unchecked:
            let _ = self.remove(keys[c.id])

    /// How many copies of `x` are currently in the bag.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `SortedBag.of([2, 2, 3]).count(2) == 2`
    /// example: `SortedBag.of([2, 2, 3]).count(4) == 0`
    pub fn count(self, x: K) -> Int:
        let s = self.ge(x, true)
        unchecked:
            return if s >= 0 and not (x < keys[s]): ws[s] else: 0

    /// Whether at least one copy of `x` is in the bag.
    ///
    /// `x in b` calls this.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `SortedBag.of([2, 3]).contains(3) == true`
    pub fn contains(self, x: K) -> Bool = self.count(x) > 0

    /// The smallest element, or `None` when empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `SortedBag<Int>().first == None`
    /// ties: `SortedBag.of([1, 2, 1]).first == Some(1)`
    /// traps: never
    /// example: `SortedBag.of([3, 1, 1]).first == Some(1)`
    @get pub fn first(self) -> ?K = if head >= 0: Some(keys[head * BAG_B]) else: None

    /// The largest element, or `None` when empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `SortedBag<Int>().last == None`
    /// ties: `SortedBag.of([3, 1, 3]).last == Some(3)`
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).last == Some(3)`
    @get pub fn last(self) -> ?K = if tail >= 0: Some(
        keys[tail * BAG_B + self.fill(tail) - 1]
    ) else: None

    /// Remove and return one copy of the smallest element, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b = SortedBag<Int>(); b.pop_first() == None`
    /// traps: never
    /// example: `var b = SortedBag.of([3, 1, 1]); b.pop_first() == Some(1) and b.count(1) == 1`
    @lint(
        pop_min_loop,
        from: "let x = self.first!\nself.remove(x)",
        to: "let x = self.pop_first()!",
    )
    pub fn pop_first(mut self) -> ?K:
        let x = self.first
        if x == None: return None
        let _ = self.remove(x!)
        x

    /// \(O(\log n)\). Removes and returns the least element satisfying monotone `ok` (false then
    /// true): "the shortest partner that completes this chain".
    ///
    /// One copy is removed. `None`, and no change, when `ok` holds for no element. It is
    /// `first_true(ok)` followed by `remove` of the result, in one descent.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b = SortedBag<Int>(); b.take_first(x => true) == None`
    /// ties: `var b = SortedBag.of([4, 4]); b.take_first(x => x >= 3) == Some(4) and b.count(4) == 1`
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `var b = SortedBag.of([1, 4, 4, 6]); b.take_first(x => x >= 3) == Some(4) and b.to_list() == [1, 4, 6]`
    /// example: `var b = SortedBag.of([1]); b.take_first(x => x >= 3) == None and b.len == 1`
    pub fn take_first(mut self, ok: K -> Bool) -> ?K:
        let x = self.first_true(ok)
        if let v = x: let _ = self.remove(v)
        x

    /// Removes the elements whose value lies in `r`, yielding them in order; each removal is a
    /// full \(O(\log n)\) `remove`.
    ///
    /// Each copy is yielded once, so a key with count `c` is yielded `c` times. A loop that
    /// stops early leaves the elements not yet yielded in the bag. `r` may be any ordered range
    /// (`a..b`, `a..=b`, `..b`, `a..`, `..`).
    ///
    /// cost: \(O(k \log n)\) expected for `k` removed elements
    /// empty: `var b = SortedBag.of([1, 2]); var e = SortedBag<Int>(); b.drain_range(2..2).collect() == [] and e.drain_range(0..9).collect() == [] and b.len == 2`
    /// order: ascending
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5, 7]); b.drain_range(3..6).collect() == [3, 3, 5] and b.to_list() == [1, 7]`
    pub fn drain_range(mut self, r: Range<K>) -> yields K:
        if r.is_empty: return
        while let x = self.range_first(Some(r.l), Some(r.r), false):
            let _ = self.remove(x)
            yield x

    /// Remove and return one copy of the largest element, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b = SortedBag<Int>(); b.pop_last() == None`
    /// traps: never
    /// example: `var b = SortedBag.of([3, 3, 1]); b.pop_last() == Some(3) and b.to_list() == [1, 3]`
    @lint(pop_min_loop, from: "let x = self.last!\nself.remove(x)", to: "let x = self.pop_last()!")
    pub fn pop_last(mut self) -> ?K:
        let x = self.last
        if x == None: return None
        let _ = self.remove(x!)
        x

    /// Number of elements strictly less than `k` (counting duplicates).
    ///
    /// `k` need not be in the bag. `b.nth(b.rank(k))` is the least element `>= k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().rank(5) == 0`
    /// traps: never
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(2) == 1`
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(3) == 3`
    pub fn rank(self, k: K) -> Int = self.below(x => x < k)

    fn rank_le(self, k: K) -> Int = self.below(x => not (k < x))

    fn lo_ok(self, x: K, lo: ?K) -> Bool:
        if let a = lo: not (x < a) else: true

    fn hi_ok(self, x: K, hi: ?K, hi_inc: Bool) -> Bool:
        if let b = hi:
            if hi_inc: not (b < x) else: x < b
        else:
            true

    /// The keys of `self` inside `r`, as an \(O(1)\) view; replaces `first_ge`, `first_gt`,
    /// `last_le`, `last_lt` and `count_range` (decision 103).
    ///
    /// `r` is any ordered range: `a..b`, `a..=b`, `..b`, `..=b`, `a..` or `..`. The view lends
    /// `self`, so `self` cannot change while it is in use.
    ///
    /// cost: \(O(1)\) to make; see `SortedRange` for each query
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).len == 0`
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(2..=7).len == 3`
    pub fn range(self, r: Range<K>) -> lends SortedRange<K> =
        SortedRange(self, Some(r.l), Some(r.r), false)

    fn range_first(self, lo: ?K, hi: ?K, hi_inc: Bool) -> ?K:
        let s = if let a = lo: self.ge(a, true) else: self.edge(false)
        if s < 0: return None
        unchecked:
            let k = keys[s]
            return if self.hi_ok(k, hi, hi_inc): k else: None

    fn range_last(self, lo: ?K, hi: ?K, hi_inc: Bool) -> ?K:
        let s = if let b = hi: self.le(b, hi_inc) else: self.edge(true)
        if s < 0: return None
        unchecked:
            let k = keys[s]
            return if self.lo_ok(k, lo): k else: None

    fn range_len(self, lo: ?K, hi: ?K, hi_inc: Bool) -> Int:
        if let a = lo:
            if let b = hi:
                return self.below_diff(x => x < a, x => if hi_inc: not (b < x) else: x < b)
        let cnt_hi = if let b = hi:
            if hi_inc: self.rank_le(b) else: self.rank(b)
        else:
            self.len
        let cnt_lo = if let a = lo: self.rank(a) else: 0
        cnt_hi - cnt_lo

    fn range_each(self, lo: ?K, hi: ?K, hi_inc: Bool) -> yields K:
        var s = if let a = lo: self.ge(a, true) else: self.edge(false)
        unchecked:
            while s >= 0:
                let k = keys[s]
                if not self.hi_ok(k, hi, hi_inc): break
                for _ in 0..ws[s]: yield k
                s = self.step(s, false)

    fn range_each_rev(self, lo: ?K, hi: ?K, hi_inc: Bool) -> yields K:
        var s = if let b = hi: self.le(b, hi_inc) else: self.edge(true)
        unchecked:
            while s >= 0:
                let k = keys[s]
                if not self.lo_ok(k, lo): break
                for _ in 0..ws[s]: yield k
                s = self.step(s, true)

    /// 0-indexed `i`-th smallest element (counting duplicates), or `None` if out of bounds.
    ///
    /// It is `b.to_list()[i]` without the list; `None` for any `i` outside `0..len`, negative
    /// included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().nth(0) == None`
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2, 1]).nth(2) == Some(2)`
    /// example: `SortedBag.of([5, 2, 2, 1]).nth(4) == None`
    pub fn nth(self, i: Int) -> ?K = self.key_at(self.slot_at(i))

    // The key at slot `s`, or `None` for -1.
    fn key_at(self, s: Int) -> ?K:
        unchecked: return if s >= 0: keys[s] else: None

    // The slot of the `i`-th smallest element, or -1.
    fn slot_at(self, i: Int) -> Int:
        if i < 0 or i >= total: return -1
        var k = i
        var n = root
        var h = height
        unchecked:
            while true:
                var j = n * BAG_B
                while k >= ws[j]:
                    k -= ws[j]
                    j += 1
                if h == 0: return j
                n = self.kid(n, j - n * BAG_B)
                h -= 1
        -1

    /// Smallest key where monotone `ok(k)` holds (`false..false, true..true`), or `None`.
    /// Checks monotonicity at the cut where `ok` turns. \(O(\log n)\) expected.
    ///
    /// `ok` is called on keys along one root-to-leaf path. Checked builds check that `ok` is
    /// false on the key just below the answer.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `SortedBag<Int>().first_true(x => true) == None`
    /// ties: `SortedBag.of([5, 4, 4]).first_true(x => x >= 3) == Some(4)`
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x > 9) == None`
    pub fn first_true(self, ok: K -> Bool) -> ?K:
        if root < 0: return None
        var n = root
        for _ in 0..height: n = self.kid(n, self.pos(n, 1, k => not ok(k)) - 1)
        let p = self.pos(n, 0, k => not ok(k))
        let s = if p < self.fill(n): n * BAG_B + p else: self.step(n * BAG_B + p - 1, false)
        unchecked: return if s >= 0 and ok(keys[s]): keys[s] else: None

    /// Largest key where monotone `ok(k)` holds (`true..true, false..false`), or `None`.
    /// Checks monotonicity at the cut where `ok` turns. \(O(\log n)\) expected.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `SortedBag<Int>().last_true(x => true) == None`
    /// ties: `SortedBag.of([1, 4, 4]).last_true(x => x <= 5) == Some(4)`
    /// traps: never; an `ok` that is not true-then-false gives an unspecified answer
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x < 1) == None`
    pub fn last_true(self, ok: K -> Bool) -> ?K:
        if root < 0: return None
        var n = root
        for _ in 0..height: n = self.kid(n, self.pos(n, 1, ok) - 1)
        let p = self.pos(n, 0, ok)
        let s = if p > 0: n * BAG_B + p - 1 else: self.step(n * BAG_B, true)
        unchecked: return if s >= 0 and ok(keys[s]): keys[s] else: None

    /// Remove and return the maximal run of elements strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest element (ascending)
    /// while `ok(k)` holds. \(O(k \log n)\) expected for `k` removed elements.
    ///
    /// The walk starts at the first element past the bound and stops at the first element
    /// where `ok` is false, which stays; each copy of a removed key is one element of the
    /// result. Giving both `after` and `before` is an error.
    ///
    /// cost: \(O(k \log n)\) expected for `k` removed elements
    /// empty: `var b = SortedBag<Int>(); b.drain(while: x => true) == []`
    /// order: ascending with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var b = SortedBag.of([1, 2, 2, 5, 8]); b.drain(after: 1, while: x => x < 5) == [2, 2] and b.to_list() == [1, 5, 8]`
    /// example: `var b = SortedBag.of([1, 2, 5, 8]); b.drain(before: 8, while: x => x > 1) == [5, 2] and b.to_list() == [1, 8]`
    /// example: `var b = SortedBag.of([1, 2, 5]); b.drain(while: x => x < 3) == [1, 2]`
    pub fn drain(mut self; after: ?K = None, before: ?K = None, while ok: K -> Bool) -> List<K>:
        assert not (
            after != None and before != None
        ), "drain takes either after: or before:, not both"
        var out: List<K> = []
        if let b = before:
            while let x = self.key_at(self.le(b, false)):
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        elif let a = after:
            while let x = self.key_at(self.ge(a, false)):
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        else:
            while let x = self.first:
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        out

    /// Cursor to the smallest key, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_first() == None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_first()!) == 2`
    pub fn cursor_first(self) -> ?BagCursor = self.cursor(self.edge(false))

    /// Cursor to the largest key, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_last() == None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_last()!) == 5`
    pub fn cursor_last(self) -> ?BagCursor = self.cursor(self.edge(true))

    /// Cursor to exact key `k`, or `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_at(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.cursor_at(5) != None and b.cursor_at(3) == None`
    pub fn cursor_at(self, k: K) -> ?BagCursor = self.cursor(self.find(k))

    /// Cursor to smallest key `>= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_ge(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_ge(3)!) == 5 and b.cursor_ge(6) == None`
    pub fn cursor_ge(self, k: K) -> ?BagCursor = self.cursor(self.ge(k, true))

    /// Cursor to smallest key `> k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_gt(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_gt(2)!) == 5 and b.cursor_gt(5) == None`
    pub fn cursor_gt(self, k: K) -> ?BagCursor = self.cursor(self.ge(k, false))

    /// Cursor to largest key `<= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_le(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_le(4)!) == 2 and b.cursor_le(1) == None`
    pub fn cursor_le(self, k: K) -> ?BagCursor = self.cursor(self.le(k, true))

    /// Cursor to largest key `< k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_lt(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_lt(5)!) == 2 and b.cursor_lt(2) == None`
    pub fn cursor_lt(self, k: K) -> ?BagCursor = self.cursor(self.le(k, false))

    /// Key stored at cursor `c`.
    ///
    /// `c` must come from this bag and no mutation may have happened since it was made.
    ///
    /// cost: \(O(1)\)
    /// traps: never in release (a stale cursor reads an unspecified key); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.cursor_first()!) == 2`
    pub fn key_of(self, c: BagCursor) -> K:
        self.check(c)
        unchecked:
            return keys[c.id]

    /// Multiplicity stored at cursor `c`.
    ///
    /// cost: \(O(1)\)
    /// traps: never in release (a stale cursor reads an unspecified count); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.count_of(b.cursor_at(2)!) == 2`
    pub fn count_of(self, c: BagCursor) -> Int:
        self.check(c)
        unchecked:
            return ws[c.id]

    /// Cursor to the next larger distinct key, or `None`.
    ///
    /// Steps over all copies of the current key at once.
    ///
    /// cost: \(O(1)\) amortized, \(O(\log n)\) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.next_cursor(b.cursor_first()!)!) == 5`
    /// example: `let b = SortedBag.of([2, 5]); b.next_cursor(b.cursor_last()!) == None`
    pub fn next_cursor(self, c: BagCursor) -> ?BagCursor:
        self.check(c)
        self.cursor(self.step(c.id, false))

    /// Cursor to the next smaller distinct key, or `None`.
    ///
    /// cost: \(O(1)\) amortized, \(O(\log n)\) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 5, 5]); b.key_of(b.prev_cursor(b.cursor_last()!)!) == 2`
    /// example: `let b = SortedBag.of([2, 5]); b.prev_cursor(b.cursor_first()!) == None`
    pub fn prev_cursor(self, c: BagCursor) -> ?BagCursor:
        self.check(c)
        self.cursor(self.step(c.id, true))

    /// Visit every distinct `(key, count)` in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().entries().collect() == []`
    /// order: ascending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries()] == [2, 5]`
    /// example: `[e.count for e in SortedBag.of([5, 2, 2]).entries()] == [2, 1]`
    pub fn entries(self) -> yields (key: K, count: Int):
        var s = self.edge(false)
        unchecked:
            while s >= 0:
                yield (key: keys[s], count: ws[s])
                s = self.step(s, false)

    /// Visit every distinct `(key, count)` in descending order.
    ///
    /// Exactly the reverse of `entries`.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().entries_rev().collect() == []`
    /// order: descending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries_rev()] == [5, 2]`
    pub fn entries_rev(self) -> yields (key: K, count: Int):
        var s = self.edge(true)
        unchecked:
            while s >= 0:
                yield (key: keys[s], count: ws[s])
                s = self.step(s, true)

    /// Visit every element, duplicates included, in ascending order; `for x in b` uses this.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().each().collect() == []`
    /// order: ascending, each key `count` times
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2]).each().collect() == [2, 2, 5]`
    pub fn each(self) -> yields K:
        var s = self.edge(false)
        unchecked:
            while s >= 0:
                for _ in 0..ws[s]: yield keys[s]
                s = self.step(s, false)

    /// Visit every element, duplicates included, in descending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().each_rev().collect() == []`
    /// order: descending, each key `count` times
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2]).each_rev().collect() == [5, 2, 2]`
    pub fn each_rev(self) -> yields K:
        var s = self.edge(true)
        unchecked:
            while s >= 0:
                for _ in 0..ws[s]: yield keys[s]
                s = self.step(s, true)

    /// Every element in ascending order, repeated by multiplicity.
    ///
    /// cost: \(O(n)\)
    /// empty: `SortedBag<Int>().to_list() == []`
    /// order: ascending, each key repeated by its count; a new `List`, keyed from 0
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).to_list() == [1, 3, 3]`
    pub fn to_list(self) -> List<K>:
        var out: List<K> = []
        for (k, c) in self.entries():
            for _ in 0..c: out.push(k)
        out

    /// Preallocate leaf storage for at least `cap` distinct keys.
    ///
    /// cost: \(O(n)\) when the storage grows (it moves the keys), \(O(1)\) otherwise
    /// traps: never
    /// example: `var b = SortedBag.of([1]); b.reserve(100); b.to_list() == [1]`
    pub fn reserve(mut self, cap: Int):
        keys.reserve(max(cap, 0) + BAG_B)
        ws.reserve(max(cap, 0) + BAG_B)

    /// Split off and return all elements `>= k`, keeping `< k` in `self`.
    ///
    /// Every copy of a key goes to the same side.
    ///
    /// The larger side keeps the node array (`self` on a tie); the smaller side's keys move to a
    /// fresh one.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's distinct keys
    /// empty: `var b = SortedBag<Int>(); let hi = b.split_at(3); b.len == 0 and hi.len == 0`
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5]); let hi = b.split_at(3); b.to_list() == [1] and hi.to_list() == [3, 3, 5]`
    pub fn split_at(mut self, k: K) -> SortedBag<K>:
        var out = SortedBag<K>.new()
        var moved: List<(K, Int)> = []
        let low = self.rank(k) < total - self.rank(k)
        if low: self.swap(mut out)
        var s = if low: out.edge(false) else: self.ge(k, true)
        unchecked:
            while s >= 0:
                let (x, c) = if low: (out.keys[s], out.ws[s]) else: (keys[s], ws[s])
                if low and not (x < k): break
                moved.push((x, c))
                s = if low: out.step(s, false) else: self.step(s, false)
        for (x, c) in moved:
            if low:
                let _ = out.remove_all(x)
                self.add_many(x, c)
            else:
                let _ = self.remove_all(x)
                out.add_many(x, c)
        self.changed(BAG_REMOVE)
        out

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The inverse of `split_at`. The precondition is checked in \(O(1)\) against `self.last` and
    /// `other.first`; either side may be empty.
    ///
    /// The smaller side's keys move into the larger side's node array (`self`'s on a tie), as in
    /// `split_at`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's distinct keys
    /// traps: when both are non-empty and `self.last >= other.first`
    /// example: `var a = SortedBag.of([1, 2]); a.join(SortedBag.of([4, 4])); a.to_list() == [1, 2, 4, 4]`
    pub fn join(mut self, var other: SortedBag<K>):
        if let a = self.last:
            if let b = other.first: assert a <
                b, "`join` needs every key of `self` below every key of `other`"
        if total < other.total: self.swap(mut other)
        for (k, c) in other.entries(): self.add_many(k, c)
        self.changed(BAG_ADD)

    /// Merge all elements of `other` into `self` (small-to-large, emptying `other`).
    ///
    /// Copies add up: a key with count `a` in `self` and `b` in `other` ends with `a + b`. The
    /// smaller bag's elements are inserted into the larger, whichever side that is (`self` on a
    /// tie).
    ///
    /// cost: \(O(\min(n, m) \log(n + m))\) expected
    /// empty: `var a = SortedBag.of([1, 2]); var e = SortedBag<Int>(); a.absorb(mut e); a.to_list() == [1, 2]`
    /// traps: never
    /// example: `var a = SortedBag.of([1, 3]); var b = SortedBag.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedBag<K>):
        if total < other.total: self.swap(mut other)
        for (k, c) in other.entries(): self.add_many(k, c)
        self.changed(BAG_ADD)
        other.clear()

    /// Sum of all elements (for additive `K`). \(O(1)\) when augmented, \(O(n)\) otherwise.
    ///
    /// Each copy is added. `BagNode` keeps no subtree sum, so this bag is never augmented and
    /// the cost is \(O(n)\); `AugSortedBag` keeps sums.
    ///
    /// cost: \(O(n)\)
    /// empty: `SortedBag<Int>().sum() == 0`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `SortedBag.of([2, 2, 5]).sum() == 9`
    pub fn sum(self) -> K:
        var t = K.zero
        for (k, c) in self.entries():
            for _ in 0..c: t = t + k
        t

    fn kid(self, n: Int, i: Int) -> Int:
        unchecked: return kids[self.kix(n) * BAG_B + i]

    // Copy entry `i` of node `a` to entry `j` of node `b`; an `inner` entry carries a child too.
    fn mv(mut self, a: Int, i: Int, b: Int, j: Int, inner: Bool):
        let s = a * BAG_B + i
        self.put(b * BAG_B + j, keys[s], ws[s], if inner or pay: self.kid(a, i) else: -1)

    fn put(mut self, s: Int, x: K, w: Int, k: Int):
        keys[s] = x
        ws[s] = w
        if k >= 0: kids[self.kix(s / BAG_B) * BAG_B + s % BAG_B] = k

    fn node(mut self, x: K, inner: Bool) -> Int:
        var u = meta.len / 4
        if free != -1:
            u = free
            free = self.nxt(u)
        else:
            // The first node grows slot by slot, so a small bag holds only its keys; any
            // later node gets all its slots, after the last one is filled out.
            if u > 0:
                while keys.len < u * BAG_B + BAG_B:
                    keys.push(x)
                    ws.push(0)
            meta.reserve(4)
            for v in [0, -1, -1, -1]: meta.push(v)
        meta[4 * u] = 0
        meta[4 * u + 1] = -1
        meta[4 * u + 2] = -1
        if (inner or pay) and self.kix(u) < 0:
            meta[4 * u + 3] = kids.len / BAG_B
            for _ in 0..BAG_B: kids.push(-1)
        u

    fn fill(self, n: Int) -> Int = meta[4 * n]

    fn nxt(self, n: Int) -> Int = meta[4 * n + 1]

    fn prv(self, n: Int) -> Int = meta[4 * n + 2]

    fn kix(self, n: Int) -> Int = meta[4 * n + 3]

    // Node `b` (or the end, -1) follows `a`; a leaf followed by nothing is the tail.
    fn link(mut self, a: Int, b: Int, inner: Bool):
        meta[4 * a + 1] = b
        if b != -1: meta[4 * b + 2] = a
        elif not inner: tail = a

    fn release(mut self, n: Int):
        meta[4 * n + 1] = free
        free = n

    fn weight(self, n: Int) -> Int = slot_sum(ws, n * BAG_B, self.fill(n))

    // The first `i` in `a..` the fill of `n` whose key fails `go`; the keys passing `go` form a prefix.
    fn at(self, n: Int, a: Int, x: K, upper: Bool) -> Int:
        var l = a
        var r = 0
        r = self.fill(n)
        while l < r:
            let m = (l + r) / 2
            let k = keys[n * BAG_B + m]
            if (if upper: not x < k else: k < x): l = m + 1 else: r = m
        l

    fn pos(self, n: Int, a: Int, go: K -> Bool) -> Int:
        var l = a
        var r = 0
        r = self.fill(n)
        while l < r:
            let m = (l + r) / 2
            let k = keys[n * BAG_B + m]
            if go(k): l = m + 1 else: r = m
        l

    // Make room for an entry at `p` of node `n`, splitting a full node in half; returns the
    // new right half (or -1) and the slot to fill.
    fn room(mut self, n: Int, inner: Bool, p: Int, x: K) -> (Int, Int):
        let f = self.fill(n)
        if f < BAG_B:
            if keys.len == n * BAG_B + f:
                // A small bag's one leaf starts at 4 slots, not 1.
                keys.reserve(4)
                ws.reserve(4)
                keys.push(x)
                ws.push(0)
            var i = f
            while i > p:
                self.mv(n, i - 1, n, i, inner)
                i -= 1
            meta[4 * n] = f + 1
            return (-1, n * BAG_B + p)
        let m = self.node(x, inner)
        let h = BAG_B / 2
        for i in h..BAG_B: self.mv(n, i, m, i - h, inner)
        meta[4 * n] = h
        meta[4 * m] = BAG_B - h
        self.link(m, self.nxt(n), inner)
        self.link(n, m, inner)
        let (_, s) = if p <= h: self.room(n, inner, p, x) else: self.room(m, inner, p - h, x)
        return (m, s)

    // Insert `c` copies of `x` under node `n` at height `h`: -2 when `unique` finds `x`, else
    // the new right sibling of `n` after a split, or -1.
    // Whether leaf `n`'s slot `p`, the first not below `x`, holds `x`.
    fn holds(self, n: Int, p: Int, x: K) -> Bool = p < self.fill(n) and not x < keys[n * BAG_B + p]

    fn ins(mut self, n: Int, h: Int, x: K, c: Int, unique: Bool) -> Int:
        if h == 0:
            let p = self.at(n, 0, x, false)
            let s = n * BAG_B + p
            if self.holds(n, p, x):
                got = s
                if unique: return -2
                ws[s] += c
                return -1
            live += 1
            let (m, t) = self.room(n, false, p, x)
            self.put(t, x, c, -1)
            got = t
            return m
        let i = self.at(n, 1, x, true) - 1
        let s = n * BAG_B + i
        let r = self.ins(self.kid(n, i), h - 1, x, c, unique)
        if r == -2: return -2
        let y = keys[s]
        if x < y: keys[s] = x
        ws[s] += c
        return if r >= 0: self.adopt(n, i, r) else: -1

    // Child `i` of inner node `n` split off `r`: link `r` after it; returns `n`'s own split, or -1.
    fn adopt(mut self, n: Int, i: Int, r: Int) -> Int:
        let w = self.weight(r)
        ws[n * BAG_B + i] -= w
        let k = keys[r * BAG_B]
        let (m, t) = self.room(n, true, i + 1, k)
        self.put(t, k, w, r)
        return m

    fn insert(mut self, x: K, c: Int, unique: Bool) -> Bool:
        if root < 0:
            root = self.node(x, false)
            height = 0
            head = root
            tail = root
        let r = self.ins(root, height, x, c, unique)
        if r == -2: return false
        if r >= 0:
            let nr = self.node(x, true)
            meta[4 * nr] = 1
            let k = keys[root * BAG_B]
            self.put(nr * BAG_B, k, total + c, root)
            root = nr
            height += 1
            let _ = self.adopt(nr, 0, r)
        total += c
        self.changed(BAG_ADD)
        true

    fn drop(mut self, n: Int, p: Int, inner: Bool):
        for i in p + 1..self.fill(n): self.mv(n, i, n, i - 1, inner)
        meta[4 * n] -= 1

    // Child `i` of inner node `n` fell below a quarter full: merge it with a neighbour, or move
    // entries over one at a time until the two are even.
    fn fix(mut self, n: Int, i: Int, inner: Bool):
        let a = if i + 1 < self.fill(n): i else: i - 1
        let x = self.kid(n, a)
        let y = self.kid(n, a + 1)
        let s = n * BAG_B + a
        var fx = self.fill(x)
        var fy = self.fill(y)
        if fx + fy <= BAG_B:
            for j in 0..fy: self.mv(y, j, x, fx + j, inner)
            meta[4 * x] = fx + fy
            ws[s] += ws[s + 1]
            self.link(x, self.nxt(y), inner)
            self.release(y)
            self.drop(n, a + 1, true)
            return
        let l = i != a
        while fx + 1 < fy or fy + 1 < fx:
            let a = if l: x else: y
            let b = if l: y else: x
            let p = if l: fx - 1 else: 0
            let q = if l: 0 else: fx
            let k = keys[a * BAG_B + p]
            let _ = self.room(b, inner, q, k)
            self.mv(a, p, b, q, inner)
            self.drop(a, p, inner)
            let d = if l: 1 else: -1
            fx -= d
            fy += d
        ws[s] = self.weight(x)
        ws[s + 1] = self.weight(y)
        keys[s + 1] = keys[y * BAG_B]

    // Remove one copy of `x` (every copy when `all`) under node `n` at height `h`; returns how
    // many were removed.
    fn del(mut self, n: Int, h: Int, x: K, all: Bool) -> Int:
        if h == 0:
            let p = self.at(n, 0, x, false)
            let s = n * BAG_B + p
            if not self.holds(n, p, x): return 0
            if not all and ws[s] > 1:
                ws[s] -= 1
                return 1
            let rem = ws[s]
            self.drop(n, p, false)
            live -= 1
            return rem
        let i = self.at(n, 1, x, true) - 1
        let u = self.kid(n, i)
        let rem = self.del(u, h - 1, x, all)
        ws[n * BAG_B + i] -= rem
        if rem > 0 and self.fill(u) < BAG_B / 4 and self.fill(n) > 1: self.fix(n, i, h > 1)
        return rem

    fn erase(mut self, x: K, all: Bool) -> Int:
        if root < 0: return 0
        let rem = self.del(root, height, x, all)
        if rem == 0: return 0
        total -= rem
        self.changed(BAG_REMOVE)
        while height > 0 and self.fill(root) == 1:
            self.release(root)
            root = self.kid(root, 0)
            height -= 1
        if self.fill(root) == 0:
            // An emptied bag gives its nodes back, as `std::set` frees each node it erases.
            let (st, p) = (stamp, pay)
            self = SortedBag<K>.new()
            stamp = st
            last_op = BAG_REMOVE
            pay = p
        rem

    fn changed(mut self, op: Int):
        stamp += 1
        last_op = op

    // The slot of the least key (the greatest when `last`), or -1.
    fn edge(self, last: Bool) -> Int:
        if root < 0: return -1
        return if last: tail * BAG_B + self.fill(tail) - 1 else: head * BAG_B

    // The slot after `s` in key order (before it when `back`), or -1.
    fn step(self, s: Int, back: Bool) -> Int:
        let l = s / BAG_B
        let p = s - l * BAG_B
        if back:
            if p > 0: return s - 1
            let q = self.prv(l)
            return if q >= 0: q * BAG_B + self.fill(q) - 1 else: -1
        if p + 1 < self.fill(l): return s + 1
        return if self.nxt(l) >= 0: self.nxt(l) * BAG_B else: -1

    // Elements whose key passes monotone `ok` (true, then false).
    fn below(self, ok: K -> Bool) -> Int = if root < 0: 0 else: self.below_in(root, height, ok)

    // `below` within node `n0` at height `h0`.
    fn below_in(self, n0: Int, h0: Int, ok: K -> Bool) -> Int:
        var t = 0
        var n = n0
        for _ in 0..h0:
            let i = self.pos(n, 1, ok) - 1
            for j in 0..i: t += ws[n * BAG_B + j]
            n = self.kid(n, i)
        for j in 0..self.pos(n, 0, ok): t += ws[n * BAG_B + j]
        t

    // `below(wide) - below(narrow)` (0 when negative) for monotone `narrow` implying `wide`: one walk
    // down the shared path, then both walks from where they fork.
    fn below_diff(self, narrow: K -> Bool, wide: K -> Bool) -> Int:
        if root < 0: return 0
        var n = root
        var h = height
        while h > 0 and self.pos(n, 1, narrow) == self.pos(n, 1, wide):
            n = self.kid(n, self.pos(n, 1, wide) - 1)
            h -= 1
        let d = self.below_in(n, h, wide) - self.below_in(n, h, narrow)
        if d > 0: d else: 0

    fn ge(self, a: K, inc: Bool) -> Int = self.bound(a, not inc, false)

    fn le(self, b: K, inc: Bool) -> Int = self.bound(b, inc, true)

    // The slot of the first key `> x` (`>= x` unless `upper`), or with `back` the last key before
    // it, or -1: `first_true` and `last_true` by comparison, so a range makes no closure.
    fn bound(self, x: K, upper: Bool, back: Bool) -> Int:
        if root < 0: return -1
        var n = root
        for _ in 0..height: n = self.kid(n, self.at(n, 1, x, upper) - 1)
        let p = self.at(n, 0, x, upper)
        let s = if back:
            if p > 0: n * BAG_B + p - 1 else: self.step(n * BAG_B, true)
        else:
            if p < self.fill(n): n * BAG_B + p else: self.step(n * BAG_B + p - 1, false)
        if s < 0: return -1
        let k = keys[s]
        return if (if upper: x < k else: not (k < x)) != back: s else: -1

    // The slot of key `k`, or -1.
    fn find(self, k: K) -> Int:
        let s = self.ge(k, true)
        unchecked: return if s >= 0 and not (k < keys[s]): s else: -1

    // In a bag made by `keyed` every leaf entry has a `kids` slot too, holding its value's index.
    fn keyed() -> SortedBag<K>:
        var b = SortedBag<K>.new()
        b.pay = true
        b

    fn vid(self, s: Int) -> Int = self.kid(s / BAG_B, s % BAG_B)

    fn set_vid(mut self, s: Int, v: Int):
        kids[self.kix(s / BAG_B) * BAG_B + s % BAG_B] = v

    fn cursor(self, s: Int) -> ?BagCursor = if s >= 0: Some(BagCursor(s, stamp)) else: None

    fn check(self, c: BagCursor):
        assert c.stamp ==
            stamp, "stale BagCursor: the bag changed ({BAG_OPS[last_op]}) after the cursor was made; make cursors after the last change"

    // Exchange contents with `o`; each side keeps its own `stamp`.
    fn swap(mut self, mut o: SortedBag<K>):
        let mine = SortedBag(
            keys.take(),
            ws.take(),
            meta.take(),
            kids.take(),
            root,
            height,
            live,
            total,
            stamp,
            last_op,
            head,
            tail,
            free,
            pay,
            got,
        )
        keys = o.keys.take()
        ws = o.ws.take()
        meta = o.meta.take()
        kids = o.kids.take()
        free = o.free
        root = o.root
        height = o.height
        live = o.live
        total = o.total
        head = o.head
        tail = o.tail
        o = mine

/// An ordered set of unique keys `K` with order statistics (`rank`, `nth`),
/// \(O(1)\) key-range views (`range(r)`), and monotone search (`first_true`/`last_true`). Backed by `SortedBag<K>`.
/// All single-key and range-summary operations are \(O(\log n)\) expected;
/// `range(r)` is \(O(1)\); range iteration is \(O(\log n + k)\);
/// `len`, `is_empty`, `clear`, `take` are \(O(1)\); `each`, `each_rev`, and `to_list` are \(O(n)\).
pub struct SortedSet<K>:
    var bag: SortedBag<K>

    /// Create an empty `SortedSet<K>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s: SortedSet<Int> = SortedSet(); s.len == 0`
    pub fn new() -> SortedSet<K> = SortedSet(SortedBag<K>.new())

    /// Build a `SortedSet` containing the distinct elements of `xs`.
    ///
    /// cost: \(O(n \log n)\) expected
    /// empty: `SortedSet<Int>.of([]).len == 0`
    /// traps: never
    /// example: `SortedSet.of([3, 1, 3]).to_list() == [1, 3]`
    pub fn of(xs: List<K>) -> SortedSet<K>:
        var s = SortedSet<K>.new()
        for x in xs: let _ = s.add(x)
        s

    /// Number of keys in the set.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedSet.of([3, 1, 3]).len == 2`
    @get pub fn len(self) -> Int:
        bag.distinct

    /// Whether the set is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedSet.of([1]).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        bag.is_empty

    /// Remove all keys from the set.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = SortedSet.of([1, 2]); s.clear(); s.len == 0`
    pub fn clear(mut self) = bag.clear()

    /// Move all keys out, leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = SortedSet.of([1, 2]); let t = s.take(); s.is_empty and t.to_list() == [1, 2]`
    pub fn take(mut self) -> SortedSet<K>:
        SortedSet(bag.take())

    /// Insert `x` if absent; returns `true` when `x` was newly inserted.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.add(2) and not s.add(2) and s.len == 2`
    pub fn add(mut self, x: K) -> Bool = bag.add_unique(x)

    /// Remove `x` if present; returns `true` if removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var s = SortedSet.of([1, 2]); s.remove(2) and not s.remove(2) and s.to_list() == [1]`
    pub fn remove(mut self, x: K) -> Bool = bag.remove(x)

    /// Add `x` when `on` is true, or remove it otherwise, matching `Set.set` and `Bitset.set`.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.set(2, true); s.set(1, false); s.to_list() == [2]`
    pub fn set(mut self, x: K, on: Bool):
        if on:
            let _ = self.add(x)
        else:
            let _ = self.remove(x)

    /// Whether `x` is in the set.
    ///
    /// `x in s` calls this.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `SortedSet.of([1, 4]).contains(4) and not SortedSet.of([1, 4]).contains(2)`
    pub fn contains(self, x: K) -> Bool = bag.contains(x)

    /// Smallest key in the set, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `SortedSet<Int>().first == None`
    /// ties: `let s = SortedSet.of([1, 1, 2]); s.first == Some(1) and s.len == 2`
    /// traps: never
    /// example: `SortedSet.of([4, 1, 9]).first == Some(1)`
    @get pub fn first(self) -> ?K:
        bag.first

    /// Largest key in the set, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `SortedSet<Int>().last == None`
    /// ties: `let s = SortedSet.of([1, 2, 2]); s.last == Some(2) and s.len == 2`
    /// traps: never
    /// example: `SortedSet.of([4, 1, 9]).last == Some(9)`
    @get pub fn last(self) -> ?K:
        bag.last

    /// Remove and return the smallest key, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s = SortedSet<Int>(); s.pop_first() == None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_first() == Some(1) and s.to_list() == [4]`
    @lint(
        pop_min_loop,
        from: "let x = self.first!\nself.remove(x)",
        to: "let x = self.pop_first()!",
    )
    pub fn pop_first(mut self) -> ?K = bag.pop_first()

    /// Remove and return the largest key, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s = SortedSet<Int>(); s.pop_last() == None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_last() == Some(4) and s.to_list() == [1]`
    @lint(pop_min_loop, from: "let x = self.last!\nself.remove(x)", to: "let x = self.pop_last()!")
    pub fn pop_last(mut self) -> ?K = bag.pop_last()

    /// Number of keys strictly less than `k`.
    ///
    /// `k` need not be in the set.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedSet<Int>().rank(5) == 0`
    /// traps: never
    /// example: `SortedSet.of([1, 4, 9]).rank(4) == 1`
    /// example: `SortedSet.of([1, 4, 9]).rank(5) == 2`
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// 0-indexed `i`-th smallest key, or `None` if out of range.
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedSet<Int>().nth(0) == None`
    /// traps: never
    /// example: `SortedSet.of([9, 1, 4]).nth(1) == Some(4)`
    /// example: `SortedSet.of([9, 1, 4]).nth(-1) == None`
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// Smallest key `k` where monotone `ok(k)` holds (`false..false, true..true`), or `None`.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `SortedSet<Int>().first_true(x => true) == None`
    /// ties: `SortedSet.of([5, 4, 4]).first_true(x => x >= 3) == Some(4)`
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `SortedSet.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    pub fn first_true(self, ok: K -> Bool) -> ?K:
        bag.first_true(ok)

    /// Largest key `k` where monotone `ok(k)` holds (`true..true, false..false`), or `None`.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `SortedSet<Int>().last_true(x => true) == None`
    /// ties: `SortedSet.of([1, 4, 4]).last_true(x => x <= 5) == Some(4)`
    /// traps: never; an `ok` that is not true-then-false gives an unspecified answer
    /// example: `SortedSet.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)`
    pub fn last_true(self, ok: K -> Bool) -> ?K:
        bag.last_true(ok)

    /// Remove and return the maximal run of keys strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest key (ascending)
    /// while `ok(k)` holds. \(O(k \log n)\) expected for `k` removed keys.
    ///
    /// The walk stops at the first key where `ok` is false, which stays. Giving both `after`
    /// and `before` is an error.
    ///
    /// cost: \(O(k \log n)\) expected for `k` removed keys
    /// empty: `var s = SortedSet<Int>(); s.drain(while: x => true) == []`
    /// order: ascending with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var s = SortedSet.of([1, 2, 5, 8]); s.drain(after: 1, while: x => x < 6) == [2, 5] and s.to_list() == [1, 8]`
    /// example: `var s = SortedSet.of([1, 2, 5, 8]); s.drain(before: 8, while: x => x > 1) == [5, 2]`
    pub fn drain(
        mut self; after: ?K = None,
        before: ?K = None,
        while ok: K -> Bool
    ) -> List<K> = bag.drain(after:, before:, while: ok)

    /// The keys of `self` inside `r`, as an \(O(1)\) view; replaces `first_ge`, `first_gt`,
    /// `last_le`, `last_lt` and `count_range` (decision 103).
    ///
    /// `r` is any ordered range: `a..b`, `a..=b`, `..b`, `..=b`, `a..` or `..`. The view lends
    /// `self`, so `self` cannot change while it is in use.
    ///
    /// cost: \(O(1)\) to make; see `SortedRange` for each query
    /// empty: `let s = SortedSet.of([1, 5]); s.range(2..5).len == 0`
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..7).last == Some(3)`
    pub fn range(self, r: Range<K>) -> lends SortedRange<K> = bag.range(r)

    fn range_first(self, lo: ?K, hi: ?K, hi_inc: Bool) -> ?K = bag.range_first(lo, hi, hi_inc)

    fn range_last(self, lo: ?K, hi: ?K, hi_inc: Bool) -> ?K = bag.range_last(lo, hi, hi_inc)

    fn range_len(self, lo: ?K, hi: ?K, hi_inc: Bool) -> Int = bag.range_len(lo, hi, hi_inc)

    fn range_each(self, lo: ?K, hi: ?K, hi_inc: Bool) -> yields K:
        for k in bag.range_each(lo, hi, hi_inc): yield k

    fn range_each_rev(self, lo: ?K, hi: ?K, hi_inc: Bool) -> yields K:
        for k in bag.range_each_rev(lo, hi, hi_inc): yield k

    /// Visit every key in ascending order.
    ///
    /// `for x in s` uses this.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedSet<Int>().each().collect() == []`
    /// order: ascending
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).each().collect() == [1, 2, 3]`
    pub fn each(self) -> yields K:
        for (k, _) in bag.entries(): yield k

    /// Visit every key in descending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedSet<Int>().each_rev().collect() == []`
    /// order: descending
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).each_rev().collect() == [3, 2, 1]`
    pub fn each_rev(self) -> yields K:
        for (k, _) in bag.entries_rev(): yield k

    /// Return all keys in ascending order as a `List<K>`.
    ///
    /// cost: \(O(n)\)
    /// empty: `SortedSet<Int>().to_list() == []`
    /// order: ascending; a new `List`, keyed from 0
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).to_list() == [1, 2, 3]`
    pub fn to_list(self) -> List<K> = bag.to_list()

    /// Preallocate leaf storage for at least `cap` keys.
    ///
    /// cost: \(O(n)\) when the storage grows (it moves the keys), \(O(1)\) otherwise
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.reserve(100); s.to_list() == [1]`
    pub fn reserve(mut self, cap: Int) = bag.reserve(cap)

    /// Split off and return all keys `>= k`, keeping `< k` in `self`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's size
    /// empty: `var s = SortedSet<Int>(); let hi = s.split_at(3); s.len == 0 and hi.len == 0`
    /// traps: never
    /// example: `var s = SortedSet.of([1, 3, 5]); let hi = s.split_at(3); s.to_list() == [1] and hi.to_list() == [3, 5]`
    pub fn split_at(mut self, k: K) -> SortedSet<K> = SortedSet(bag.split_at(k))

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The precondition is checked in \(O(1)\) against `self.last` and `other.first`.
    ///
    /// cost: as `SortedBag.join`
    /// traps: when both are non-empty and `self.last >= other.first`
    /// example: `var a = SortedSet.of([1, 2]); a.join(SortedSet.of([4])); a.to_list() == [1, 2, 4]`
    pub fn join(mut self, other: SortedSet<K>) = bag.join(other.bag)

    /// Merge all keys of `other` into `self` (small-to-large, emptying `other`).
    ///
    /// A key in both sets appears once.
    ///
    /// cost: \(O(\min(n, m) \log(n + m))\) expected
    /// empty: `var a = SortedSet.of([1, 2]); var e = SortedSet<Int>(); a.absorb(mut e); a.to_list() == [1, 2]`
    /// traps: never
    /// example: `var a = SortedSet.of([1, 3]); var b = SortedSet.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedSet<K>):
        if bag.distinct < other.bag.distinct:
            let small = bag.take()
            bag = other.bag.take()
            other.bag = small
        for (k, _) in other.bag.entries(): let _ = bag.add_unique(k)
        other.bag.clear()

/// Order-statistic ordered map from `K` to `V`, each value beside its key in the `SortedBag` B+tree (decision 579).
/// `get`, `set`, `remove`, `rank`, `nth`, `first_ge`/`last_le`, `first_true`/`last_true`, `split_at`, `join` are \(O(\log n)\).
///
/// Entries are ordered by key; keys are distinct, so no lookup has a tie to break. Every
/// entry-returning method gives `(key, val)`.
pub struct SortedMap<K, V>:
    ks: SortedBag<K>
    vals: List<V>
    vfree: List<Int>

    fn val(self, s: Int) -> V = vals[ks.vid(s)]

    fn entry(self, s: Int) -> ?(K, V):
        unchecked: return if s >= 0: (ks.keys[s], self.val(s)) else: None

    fn at(self, c: ?BagCursor) -> ?(K, V) = self.entry(if let x = c: x.id else: -1)

    fn swap(mut self, mut o: SortedMap<K, V>):
        let mine = SortedMap(ks.take(), vals.take(), vfree.take())
        ks = o.ks.take()
        vals = o.vals.take()
        vfree = o.vfree.take()
        o = mine

    /// Create an empty `SortedMap<K, V>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.len == 0`
    pub fn new() -> SortedMap<K, V>:
        SortedMap(SortedBag<K>.keyed(), [], [])

    /// Preallocate key and value storage for at least `cap` entries.
    ///
    /// cost: \(O(n)\) when the storage grows (it moves the keys), \(O(1)\) otherwise
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.reserve(100); m.get(1) == Some("a")`
    pub fn reserve(mut self, cap: Int):
        ks.reserve(cap)
        vals.reserve(cap)

    /// Number of key-value pairs.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(1, "b"); m.len == 1`
    @get pub fn len(self) -> Int:
        ks.len

    /// Whether the map is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.is_empty`
    @get pub fn is_empty(self) -> Bool:
        ks.is_empty

    /// Remove all entries.
    ///
    /// The key and value lists keep their capacity.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.clear(); m.get(1) == None`
    pub fn clear(mut self):
        ks.clear()
        vals.clear()
        vfree.clear()

    /// Move contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); let t = m.take(); m.is_empty and t.get(1) == Some("a")`
    pub fn take(mut self) -> SortedMap<K, V>:
        SortedMap(ks.take(), vals.take(), vfree.take())

    /// Look up `k`, returning `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.get(1) == Some("a") and m.get(2) == None`
    pub fn get(self, k: K) -> ?V:
        let s = ks.find(k)
        if s >= 0: self.val(s) else: None

    /// Whether `k` is present.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.contains(1) and not m.contains(2)`
    pub fn contains(self, k: K) -> Bool:
        ks.find(k) >= 0

    /// Insert or overwrite `k -> v` (W8 `IndexSet` backing `m[k] = v`).
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m[1] = "a"; m[1] = "b"; m.get(1) == Some("b")`
    pub fn set(mut self, k: K, v: V):
        if not ks.insert(k, 1, true):
            vals[ks.vid(ks.got)] = v
            return
        var i = vals.len
        if let f = vfree.pop():
            i = f
            vals[i] = v
        else: vals.push(v)
        ks.set_vid(ks.got, i)

    /// Remove `k` and return its previous value, or `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.remove(1) == Some("a") and m.remove(1) == None`
    pub fn remove(mut self, k: K) -> ?V:
        let s = ks.find(k)
        if s < 0: return None
        let i = ks.vid(s)
        let v = vals[i]
        vfree.push(i)
        let _ = ks.remove(k)
        v

    /// Smallest `(key, val)` pair, or `None` if empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(1, "b"); m.first == Some((1, "b")) and m.len == 1`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.first == Some((1, "a"))`
    @get pub fn first(self) -> ?(K, V):
        self.entry(ks.edge(false))

    /// Largest `(key, val)` pair, or `None` if empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(2, "a"); m.set(2, "b"); m.last == Some((2, "b")) and m.len == 1`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.last == Some((3, "c"))`
    @get pub fn last(self) -> ?(K, V):
        self.entry(ks.edge(true))

    /// Remove and return the smallest `(key, val)` pair, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.pop_first() == None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.pop_first() == Some((1, "a")) and m.len == 1`
    pub fn pop_first(mut self) -> ?(K, V):
        let e = self.entry(ks.edge(false))
        if let (k, _) = e: let _ = self.remove(k)
        e

    /// Remove and return the largest `(key, val)` pair, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.pop_last() == None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.pop_last() == Some((3, "c")) and m.len == 1`
    pub fn pop_last(mut self) -> ?(K, V):
        let e = self.entry(ks.edge(true))
        if let (k, _) = e: let _ = self.remove(k)
        e

    /// Smallest entry with key `>= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first_ge(0) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.first_ge(2) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.first_ge(2) == Some((3, "c")) and m.first_ge(3) == Some((3, "c"))`
    pub fn first_ge(self, k: K) -> ?(K, V):
        self.at(ks.cursor_ge(k))

    /// Smallest entry with key `> k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first_gt(0) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.first_gt(2) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.first_gt(1) == Some((3, "c")) and m.first_gt(3) == None`
    pub fn first_gt(self, k: K) -> ?(K, V):
        self.at(ks.cursor_gt(k))

    /// Largest entry with key `<= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last_le(9) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.last_le(4) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.last_le(2) == Some((1, "a")) and m.last_le(0) == None`
    pub fn last_le(self, k: K) -> ?(K, V):
        self.at(ks.cursor_le(k))

    /// Largest entry with key `< k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last_lt(9) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.last_lt(4) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.last_lt(3) == Some((1, "a")) and m.last_lt(1) == None`
    pub fn last_lt(self, k: K) -> ?(K, V):
        self.at(ks.cursor_lt(k))

    /// Number of entries with key strictly less than `k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.rank(1) == 0`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.rank(3) == 1 and m.rank(9) == 2`
    pub fn rank(self, k: K) -> Int:
        ks.rank(k)

    /// The `i`-th entry in key order (0-based), or `None`.
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.nth(0) == None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.nth(1) == Some((3, "c")) and m.nth(2) == None`
    pub fn nth(self, i: Int) -> ?(K, V):
        self.entry(ks.slot_at(i))

    /// Smallest entry where monotone `ok(k, v)` becomes true, or `None`.
    ///
    /// `ok` must be false-then-true along ascending keys.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first_true((k, v) => true) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(5, "a"); m.set(4, "b"); m.first_true((k, v) => k >= 3) == Some((4, "b"))`
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `var m: SortedMap<Int, Int> = SortedMap(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.first_true((k, v) => v >= 30) == Some((4, 40))`
    pub fn first_true(self, ok: (K, V) -> Bool) -> ?(K, V):
        var l = 0
        var r = ks.len
        while l < r:
            let m = (l + r) / 2
            let s = ks.slot_at(m)
            if ok(ks.keys[s], self.val(s)): r = m else: l = m + 1
        self.entry(ks.slot_at(l))

    /// Largest entry where monotone `ok(k, v)` still holds, or `None`.
    ///
    /// `ok` must be true-then-false along ascending keys.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last_true((k, v) => true) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(4, "b"); m.last_true((k, v) => k <= 5) == Some((4, "b"))`
    /// traps: never; an `ok` that is not true-then-false gives an unspecified answer
    /// example: `var m: SortedMap<Int, Int> = SortedMap(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.last_true((k, v) => k <= 5) == Some((4, 40))`
    pub fn last_true(self, ok: (K, V) -> Bool) -> ?(K, V):
        var l = 0
        var r = ks.len
        while l < r:
            let m = (l + r) / 2
            let s = ks.slot_at(m)
            if ok(ks.keys[s], self.val(s)): l = m + 1 else: r = m
        self.entry(ks.slot_at(l - 1))

    /// Remove and return entries after `after` (ascending) or before `before` (descending) while `ok` holds.
    ///
    /// With neither bound the walk starts at the smallest key. It stops at the first entry
    /// where `ok` is false, which stays; bounds are exclusive. Giving both is an error.
    ///
    /// cost: \(O(k \log n)\) expected for `k` removed entries
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.drain(while: (k, v) => true) == []`
    /// order: ascending key with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var m: SortedMap<Int, Int> = SortedMap(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.drain(after: 1, while: (k, v) => v < 50) == [(4, 40)] and m.len == 2`
    pub fn drain(
        mut self; after: ?K = None,
        before: ?K = None,
        while ok: (K, V) -> Bool
    ) -> List<(K, V)>:
        assert not (
            after != None and before != None
        ), "drain takes either after: or before:, not both"
        var out: List<(K, V)> = []
        while true:
            let s = if let b = before: ks.le(b, false) elif let a = after: ks.ge(
                a,
                false,
            ) else: ks.edge(false)
            let (k, v) = self.entry(s) ?? break
            if not ok(k, v): break
            let _ = self.remove(k)
            out.push((k, v))
        out

    /// Split off and return all entries with key `>= k`.
    ///
    /// `self` keeps the entries with key `< k`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected for the keys, plus \(O(m)\) expected to move the `m` values split off
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); let hi = m.split_at(3); m.len == 0 and hi.len == 0`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); let hi = m.split_at(2); m.len == 1 and hi.first == Some((3, "c"))`
    pub fn split_at(mut self, k: K) -> SortedMap<K, V>:
        var out = SortedMap<K, V>.new()
        let r = ks.rank(k)
        if r < ks.len - r:
            self.swap(mut out)
            while let (x, v) = out.first and x < k:
                let _ = out.remove(x)
                self.set(x, v)
        else:
            while let (x, v) = self.last and not (x < k):
                let _ = self.remove(x)
                out.set(x, v)
        out

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The precondition is checked in \(O(1)\) against the last key of `self` and the first of
    /// `other`.
    ///
    /// cost: as `SortedBag.join` for the keys, plus \(O(m)\) expected to move the `m` values of `other`
    /// traps: when both are non-empty and the last key of `self` is `>=` the first key of `other`
    /// example: `var a: SortedMap<Int, Str> = SortedMap(); a.set(1, "a"); var b: SortedMap<Int, Str> = SortedMap(); b.set(3, "c"); a.join(b); a.len == 2`
    pub fn join(mut self, var other: SortedMap<K, V>):
        if let (a, _) = self.last:
            if let (
                b,
                _,
            ) = other.first: assert a <
                b, "`join` needs every key of `self` below every key of `other`"
        if self.len < other.len: self.swap(mut other)
        for (x, v) in other.items: self.set(x, v)

    /// Merge all entries of `other` into `self` (emptying `other`), combining collisions with `join` or overwriting.
    ///
    /// For a key in both maps the result is `join(self's value, other's value)`, or `other`'s
    /// value when `join` is `None`, whichever map is drained into the other.
    ///
    /// cost: \(O(\min(n, m) \log(n + m))\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); var e: SortedMap<Int, Str> = SortedMap(); m.absorb(mut e); m.len == 1 and m.first == Some((1, "a"))`
    /// traps: never
    /// example: `var a: SortedMap<Int, Int> = SortedMap(); a.set(1, 10); var b: SortedMap<Int, Int> = SortedMap(); b.set(1, 5); b.set(2, 7); a.absorb(mut b, join: (x, y) => x + y); a.get(1) == Some(15) and a.get(2) == Some(7) and b.is_empty`
    pub fn absorb(mut self, mut other: SortedMap<K, V>, join: ?((V, V) -> V) = None):
        var small = other.take()
        var flip = false
        if small.len > self.len:
            self.swap(mut small)
            flip = true
        for (k, v) in small.items:
            let s = ks.find(k)
            if s < 0:
                self.set(k, v)
                continue
            let i = ks.vid(s)
            if let j = join: vals[i] = if flip: j(v, vals[i]) else: j(vals[i], v)
            elif not flip: vals[i] = v

    /// \(O(1)\) to make. Yields `(key, val)` pairs in ascending key order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.items.collect() == []`
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.items.collect() == [(1, "a"), (3, "c")]`
    @get pub fn items(self) -> yields (K, V):
        var s = ks.edge(false)
        unchecked:
            while s >= 0:
                yield (ks.keys[s], self.val(s))
                s = ks.step(s, false)

    /// Yield `(key, val)` pairs in descending key order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.each_rev().collect() == []`
    /// order: descending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.each_rev().collect() == [(3, "c"), (1, "a")]`
    pub fn each_rev(self) -> yields (K, V):
        var s = ks.edge(true)
        unchecked:
            while s >= 0:
                yield (ks.keys[s], self.val(s))
                s = ks.step(s, true)

    /// The map's keys, in ascending order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.keys.collect() == []`
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.keys.collect() == [1, 3]`
    @get pub fn keys(self) -> yields K:
        for k in ks.entries(): yield k.key

    /// The map's values, in ascending order of their keys.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.values.collect() == []`
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.values.collect() == ["a", "c"]`
    @get pub fn values(self) -> yields V:
        var s = ks.edge(false)
        unchecked:
            while s >= 0:
                yield self.val(s)
                s = ks.step(s, false)

/// Monoid-augmented ordered multiset maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Supports \(O(\log n)\) `add`, `remove`, `rank`, `nth`, `fold_lt`, `fold_le`, `fold_ge`, `fold_gt`, `fold_between`, and \(O(1)\) `all`.
///
/// Every fold combines `leaf(k)` over the chosen elements in ascending key order, one `leaf(k)`
/// per copy, starting from `id`; `op` need not be commutative. An empty selection folds to `id`.
pub struct AugSortedBag<K, T>:
    bag: SortedBag<K>
    op: (T, T) -> T
    id: T
    leaf: K -> T
    // The fold of each node's subtree, by node id.
    agg: List<T> = []

    // `c` copies of `v` folded together, by doubling.
    fn times(self, v: T, c: Int) -> T:
        if c == 1: return v
        let h = self.times(v, c / 2)
        let d = op(h, h)
        if c % 2 == 1: op(d, v) else: d

    // The fold of entry `j` of node `n` at height `h`: a child's fold, or `leaf(k)` per copy.
    fn part(self, n: Int, h: Int, j: Int) -> T:
        let s = n * BAG_B + j
        unchecked:
            return if h > 0: agg[bag.kid(n, j)] else: self.times(leaf(bag.keys[s]), bag.ws[s])

    // The fold of entries `a..b` of node `n` at height `h`; it starts from the first entry, so
    // only an empty span reads `id`.
    fn fold_of(self, n: Int, h: Int, a: Int, b: Int) -> T:
        if a >= b: return id
        var acc = self.part(n, h, a)
        for j in a + 1..b: acc = op(acc, self.part(n, h, j))
        acc

    fn node_fold(self, n: Int, h: Int) -> T = self.fold_of(n, h, 0, bag.fill(n))

    fn route(self, n: Int, x: K) -> Int = bag.at(n, 1, x, true) - 1

    // Whether an edit at `x` may split (`grow`) or merge and even out nodes on its path, which
    // then touches the path's neighbours as well.
    fn tight(self, x: K, grow: Bool) -> Bool:
        if bag.root < 0: return false
        var n = bag.root
        unchecked:
            if grow and bag.fill(n) >= BAG_B: return true
            for _ in 0..bag.height:
                n = bag.kid(n, self.route(n, x))
                if (if grow: bag.fill(n) >= BAG_B else: bag.fill(n) <= BAG_B / 4): return true
        false

    // Refold the path to `x` below node `n` at height `h`, with its neighbours when `wide`.
    fn refold_at(mut self, n: Int, h: Int, x: K, wide: Bool):
        if h > 0:
            let i = self.route(n, x)
            self.refold_at(bag.kid(n, i), h - 1, x, wide)
            if wide:
                if i > 0:
                    let l = bag.kid(n, i - 1)
                    agg[l] = self.node_fold(l, h - 1)
                if i + 1 < bag.fill(n):
                    let r = bag.kid(n, i + 1)
                    agg[r] = self.node_fold(r, h - 1)
        agg[n] = self.node_fold(n, h)

    fn refold(mut self, x: K, wide: Bool):
        while agg.len < bag.meta.len / 4: agg.push(id)
        if bag.root >= 0: self.refold_at(bag.root, bag.height, x, wide)

    // The fold of the keys `< x` (`<= x` with `le`) under node `n` at height `h`.
    fn prefix(self, n: Int, h: Int, x: K, le: Bool) -> T:
        if h == 0: return self.fold_of(n, 0, 0, bag.at(n, 0, x, le))
        let i = self.route(n, x)
        let below = self.prefix(bag.kid(n, i), h - 1, x, le)
        return if i == 0: below else: op(self.fold_of(n, h, 0, i), below)

    // The fold of the keys `>= x` (`> x` with `gt`) under node `n` at height `h`.
    fn suffix(self, n: Int, h: Int, x: K, gt: Bool) -> T:
        let f = bag.fill(n)
        if h == 0: return self.fold_of(n, 0, bag.at(n, 0, x, gt), f)
        let i = self.route(n, x)
        let below = self.suffix(bag.kid(n, i), h - 1, x, gt)
        return if i + 1 >= f: below else: op(below, self.fold_of(n, h, i + 1, f))

    /// Create an empty `AugSortedBag<K, T>` with associative `op`, identity `id`, and key map `leaf`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.all == 0`
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedBag<K, T> = AugSortedBag<K, T>(
        bag: SortedBag<K>.new(),
        op:,
        id:,
        leaf:,
    )

    /// Preallocate leaf storage for at least `cap` distinct keys.
    ///
    /// cost: \(O(n)\) when the storage grows (it moves the keys), \(O(1)\) otherwise
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.reserve(100); b.len == 0`
    pub fn reserve(mut self, cap: Int):
        assert cap >= 0
        bag.reserve(cap)

    /// Total number of elements (with multiplicity).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.len == 2`
    @get pub fn len(self) -> Int = bag.len

    /// Whether the bag is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.is_empty`
    @get pub fn is_empty(self) -> Bool = bag.is_empty

    /// Fold of all elements in the bag.
    ///
    /// cost: \(O(1)\)
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.all == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.all == 7`
    @get pub fn all(self) -> T = if bag.root < 0: id else: agg[bag.root]

    /// Insert one copy of `x`.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.count(3) == 2`
    pub fn add(mut self, x: K):
        let wide = self.tight(x, true)
        let _ = bag.add(x)
        self.refold(x, wide)

    /// Remove one copy of `x`; returns false if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.remove(3) and b.all == 4 and not b.remove(2)`
    pub fn remove(mut self, x: K) -> Bool:
        let wide = self.tight(x, false)
        if not bag.remove(x): return false
        self.refold(x, wide)
        true

    /// Multiplicity of `x`.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.count(3) == 2 and b.count(1) == 0`
    pub fn count(self, x: K) -> Int = bag.count(x)

    /// Number of elements strictly less than `k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.rank(3) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.rank(3) == 1`
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// The `i`-th smallest element (0-based), or `None`.
    ///
    /// Copies count; `None` for any `i` outside `0..len`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.nth(0) == None`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.nth(2) == Some(3) and b.nth(3) == None`
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// Smallest element, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.first == None`
    /// ties: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(1); b.first == Some(1)`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.first == Some(1)`
    @get pub fn first(self) -> ?K = bag.first

    /// Largest element, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.last == None`
    /// ties: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.last == Some(3)`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.last == Some(3)`
    @get pub fn last(self) -> ?K = bag.last

    /// Fold elements with key `< hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_lt(5) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_lt(3) == 1`
    pub fn fold_lt(self, hi: K) -> T = if bag.root < 0: id else: self.prefix(
        bag.root,
        bag.height,
        hi,
        false,
    )

    /// Fold elements with key `<= hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_le(5) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_le(3) == 7`
    pub fn fold_le(self, hi: K) -> T = if bag.root < 0: id else: self.prefix(
        bag.root,
        bag.height,
        hi,
        true,
    )

    /// Fold elements with key `>= lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_ge(0) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_ge(2) == 6`
    pub fn fold_ge(self, lo: K) -> T = if bag.root < 0: id else: self.suffix(
        bag.root,
        bag.height,
        lo,
        false,
    )

    /// Fold elements with key `> lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_gt(0) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_gt(1) == 6 and b.fold_gt(3) == 0`
    pub fn fold_gt(self, lo: K) -> T = if bag.root < 0: id else: self.suffix(
        bag.root,
        bag.height,
        lo,
        true,
    )

    /// Fold the elements with key in `keys`, in ascending key order, one `leaf(k)` per copy;
    /// the one fold for any ordered range (`a..b`, `a..=b`, `..b`, `a..`), as `range(r)` is on
    /// `SortedBag` (decision 103).
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); let e = b.fold(0..9); b.add(3); e == 0 and b.fold(3..3) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold(..3) == 1 and b.fold(2..=3) == 6`
    pub fn fold(self, keys: Range<K>) -> T:
        if keys.is_empty: return id
        self.fold_between(keys.l, keys.r)

    /// Fold elements with key in `[lo, hi)`.
    ///
    /// `lo >= hi` selects nothing and gives `id`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); let e = b.fold_between(0, 9); b.add(3); e == 0 and b.fold_between(4, 2) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_between(1, 3) == 1 and b.fold_between(2, 4) == 6`
    pub fn fold_between(self, lo: K, hi: K) -> T:
        if bag.root < 0 or not (lo < hi): return id
        var n = bag.root
        var h = bag.height
        unchecked:
            while h > 0:
                let i = self.route(n, lo)
                let j = self.route(n, hi)
                if i != j:
                    var acc = self.suffix(bag.kid(n, i), h - 1, lo, false)
                    if j > i + 1: acc = op(acc, self.fold_of(n, h, i + 1, j))
                    return op(acc, self.prefix(bag.kid(n, j), h - 1, hi, false))
                n = bag.kid(n, i)
                h -= 1
            return self.fold_of(n, 0, bag.at(n, 0, lo, false), bag.at(n, 0, hi, false))

    /// Yield all elements in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.each().collect() == []`
    /// order: ascending, each key repeated by its count
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.each().collect() == [1, 3, 3]`
    pub fn each(self) -> yields K:
        for (k, c) in bag.entries():
            for _ in 0..c: yield k

pub struct AugSortedSet<K, T>:
    _len: Int
    bag: AugSortedBag<K, T>

    /// Create an empty `AugSortedSet<K, T>` with associative `op`, identity `id`, and key map `leaf`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.all == 0`
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedSet<K, T> = AugSortedSet<K, T>(
        _len: 0,
        bag: AugSortedBag(op, id, leaf),
    )

    /// Preallocate leaf storage for at least `cap` keys.
    ///
    /// cost: \(O(n)\) when the storage grows (it moves the keys), \(O(1)\) otherwise
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.reserve(100); s.len == 0`
    pub fn reserve(mut self, cap: Int) = bag.reserve(cap)

    /// Number of distinct keys.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(3); s.len == 1`
    @get pub fn len(self) -> Int = bag.len

    /// Whether the set is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.is_empty`
    @get pub fn is_empty(self) -> Bool = bag.is_empty

    /// Fold of all keys in the set.
    ///
    /// cost: \(O(1)\)
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.all == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.add(3); s.all == 4`
    @get pub fn all(self) -> T = bag.all

    /// Insert `x`; returns true if `x` was new.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3) and not s.add(3)`
    pub fn add(mut self, x: K) -> Bool:
        if bag.count(x) > 0: return false
        bag.add(x)
        true

    /// Remove `x`; returns true if `x` was present.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.remove(3) and not s.remove(3) and s.all == 0`
    pub fn remove(mut self, x: K) -> Bool = bag.remove(x)

    /// Whether `x` is present.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.contains(3) and not s.contains(1)`
    pub fn contains(self, x: K) -> Bool = bag.count(x) > 0

    /// Number of keys strictly less than `k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.rank(3) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.rank(3) == 1 and s.rank(4) == 2`
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// The `i`-th smallest key (0-based), or `None`.
    ///
    /// `None` for any `i` outside `0..len`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.nth(0) == None`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.nth(1) == Some(3) and s.nth(2) == None`
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// Smallest key, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.first == None`
    /// ties: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.add(1); s.first == Some(1) and s.len == 2`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.first == Some(1)`
    @get pub fn first(self) -> ?K = bag.first

    /// Largest key, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.last == None`
    /// ties: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.add(3); s.last == Some(3) and s.len == 2`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.last == Some(3)`
    @get pub fn last(self) -> ?K = bag.last

    /// Fold keys `< hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.fold_lt(5) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_lt(5) == 4`
    pub fn fold_lt(self, hi: K) -> T = bag.fold_lt(hi)

    /// Fold keys `<= hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.fold_le(5) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_le(5) == 9`
    pub fn fold_le(self, hi: K) -> T = bag.fold_le(hi)

    /// Fold keys `>= lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.fold_ge(0) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_ge(3) == 8`
    pub fn fold_ge(self, lo: K) -> T = bag.fold_ge(lo)

    /// Fold keys `> lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.fold_gt(0) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_gt(3) == 5`
    pub fn fold_gt(self, lo: K) -> T = bag.fold_gt(lo)

    /// Fold the keys in `keys`, in ascending order; the one fold for any ordered range (`a..b`,
    /// `a..=b`, `..b`, `a..`), so `s.fold(..b)` is a prefix sum when `op` is `+`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); let e = s.fold(0..9); s.add(3); e == 0 and s.fold(3..3) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold(..5) == 4 and s.fold(3..) == 8`
    pub fn fold(self, keys: Range<K>) -> T = bag.fold(keys)

    /// Fold keys in `[lo, hi)`.
    ///
    /// `lo >= hi` selects nothing and gives `id`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); let e = s.fold_between(0, 9); s.add(3); e == 0 and s.fold_between(4, 2) == 0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_between(1, 5) == 4 and s.fold_between(4, 2) == 0`
    pub fn fold_between(self, lo: K, hi: K) -> T = bag.fold_between(lo, hi)

    /// Yield all keys in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.each().collect() == []`
    /// order: ascending
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.each().collect() == [1, 3]`
    pub fn each(self) -> yields K:
        for x in bag.each(): yield x

/// Counts over an integer value range `r`, with \(O(1)\) point add and \(O(\sqrt{N})\) k-th
/// order statistic: the value-range frequency table for Mo's algorithm.
///
/// `N` is `r.len`; the table keeps one count per value and one per block of `2^s` values,
/// the least power of two `>= sqrt(N)`, so locating a value's block is a shift. Writes outside
/// `r` trap; reads outside `r` see a count of 0.
pub struct BucketCount:
    lo: Int
    hi: Int
    sh: Int
    total: Int
    cnt: List<Int>
    blk: List<Int>

    /// \(O(r.\mathit{len})\). Create an empty bucketed frequency table over `r`.
    ///
    /// cost: \(O(r.\mathit{len})\)
    /// empty: `var b = BucketCount(0..0); b.add(0); b.len == 0` (traps R0401)
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.len == 0`
    pub fn new(r: Range<Int>) -> BucketCount:
        let n = max(r.len, 0)
        // blocks of 2^sh >= sqrt(N) values, so a value's block is a shift, not a division
        var sh = 0
        while 1 << sh < ceil_sqrt(n): sh += 1
        BucketCount(r.start, r.start + n, sh, 0, List.filled(n, 0), List.filled((n >> sh) + 1, 0))

    /// \(O(1)\). Add `delta` (default 1; may be negative) copies of `x`.
    ///
    /// A negative `delta` removes copies; the count of `x` may not go below 0.
    ///
    /// cost: \(O(1)\)
    /// traps: when `x` is not in `r`; when the count of `x` would become negative
    /// example: `var b = BucketCount(0..10); b.add(3); b.add(7, 2); b.count(7) == 2 and b.len == 3`
    pub fn add(mut self, x: Int, delta: Int = 1):
        assert lo <= x < hi, "`x` is outside the table's range"
        let i = x - lo
        assert cnt[i] + delta >= 0, "the count of `x` would become negative"
        cnt[i] += delta
        blk[i >> sh] += delta
        total += delta

    /// \(O(1)\). Remove `delta` (default 1) copies of `x`.
    ///
    /// The same as `add(x, -delta)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `x` is not in `r`; when the count of `x` would become negative
    /// example: `var b = BucketCount(0..10); b.add(7, 2); b.remove(7); b.count(7) == 1`
    pub fn remove(mut self, x: Int, delta: Int = 1):
        self.add(x, -delta)

    /// \(O(1)\). Multiplicity of `x`.
    ///
    /// cost: \(O(1)\)
    /// traps: never; an `x` outside `r` has count 0
    /// example: `var b = BucketCount(0..10); b.add(3); b.count(3) == 1 and b.count(4) == 0 and b.count(99) == 0`
    pub fn count(self, x: Int) -> Int:
        if lo <= x < hi: cnt[x - lo] else: 0

    /// \(O(\sqrt{N})\). Number of elements strictly less than `x`.
    ///
    /// Any `x` works: below `r` gives 0, at or above `r.end` gives `len`.
    ///
    /// cost: \(O(\sqrt{N})\)
    /// empty: `BucketCount(0..10).count_lt(5) == 0`
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(3); b.add(7, 2); b.count_lt(7) == 1 and b.count_lt(8) == 3`
    pub fn count_lt(self, x: Int) -> Int:
        if x <= lo: return 0
        if x >= hi: return total
        let i = x - lo
        sum(blk[b] for b in 0..i >> sh) + sum(cnt[j] for j in (i >> sh) << sh..i)

    /// \(O(\sqrt{N})\). The `k`-th smallest element, 0-based like `nth`: `kth(0)` is the smallest.
    ///
    /// Copies count: with elements `3, 7, 7`, `kth(1)` and `kth(2)` are both `7`.
    ///
    /// cost: \(O(\sqrt{N})\)
    /// ties: `var b = BucketCount(0..10); b.add(7, 2); b.kth(0) == 7 and b.kth(1) == 7`
    /// traps: when `k` is not in `0..len`
    /// example: `var b = BucketCount(0..10); b.add(3); b.add(7, 2); b.kth(0) == 3 and b.kth(1) == 7 and b.kth(2) == 7`
    pub fn kth(self, k: Int) -> Int:
        assert 0 <= k < total, "`k` is outside `0..len`"
        var left = k
        var b = 0
        while left >= blk[b]:
            left -= blk[b]
            b += 1
        var j = b << sh
        while left >= cnt[j]:
            left -= cnt[j]
            j += 1
        lo + j

    /// The total count of everything added, with multiplicity.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(4, 3); b.len == 3`
    @get pub fn len(self) -> Int:
        total

    /// \(O(1)\). Whether the table is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(4); b.remove(4); b.is_empty`
    @get pub fn is_empty(self) -> Bool:
        total == 0

    /// Visit every stored copy, skipping empty blocks.
    ///
    /// cost: \(O(\sqrt{N} + \sqrt{N} \cdot b + \mathit{len})\) for `b` blocks holding a copy
    /// empty: `BucketCount(0..10).each().collect() == []`
    /// order: ascending value, each value repeated by its count
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(7, 2); b.add(3); b.each().collect() == [3, 7, 7]`
    pub fn each(self) -> yields Int:
        for k in 0..blk.len:
            if blk[k] == 0: continue
            for i in k << sh..min((k + 1) << sh, cnt.len):
                for _ in 0..cnt[i]: yield lo + i

    /// \(O(r.\mathit{len})\). Reset all counts to zero.
    ///
    /// cost: \(O(r.\mathit{len})\)
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(4); b.clear(); b.count(4) == 0 and b.len == 0`
    pub fn clear(mut self):
        for i in 0..cnt.len: cnt[i] = 0
        for b in 0..blk.len: blk[b] = 0
        total = 0

@test
fn split_and_join_round_trip_against_sorted_lists():
    for _ in 0..200:
        let xs = [rand(0, 20) for _ in 0..rand(0, 30)].sorted()
        let k = rand(-1, 21)
        var b = SortedBag.of(xs)
        let hi = b.split_at(k)
        assert b.to_list() == [x for x in xs if x < k] and hi.to_list() == [x for x in xs if x >= k]
        b.join(hi)
        assert b.to_list() == xs and b.len == xs.len
        var lo = SortedBag.of([x for x in xs if x < k])
        lo.join(SortedBag.of([x for x in xs if x >= k]))
        assert lo.to_list() == xs and lo.first == xs.first and lo.last == xs.last
        var s = SortedSet.of(xs)
        let sh = s.split_at(k)
        s.join(sh)
        assert s.len == SortedSet.of(xs).len
        var m: SortedMap<Int, Int> = SortedMap()
        for x in xs: m.set(x, 3 * x)
        let mh = m.split_at(k)
        assert m.len + mh.len == SortedSet.of(xs).len
        for x in xs:
            assert (if x < k: m.get(x) else: mh.get(x)) == Some(3 * x)
            assert (if x < k: mh.get(x) else: m.get(x)) == None
        m.join(mh)
        for x in xs: assert m.get(x) == Some(3 * x)

@test
fn sorted_bag_basics_duplicates_and_take():
    var b = SortedBag<Int>()
    assert b.is_empty and b.len == 0
    assert b.first == None and b.last == None
    assert b.pop_first() == None and b.pop_last() == None
    assert b.rank(5) == 0 and b.nth(0) == None and b.nth(-1) == None
    assert not b.remove(5) and not b.contains(5) and b.count(5) == 0

    for x in [5, 2, 5, 1, 2, 5, 9]: assert b.add(x)
    assert b.len == 7 and not b.is_empty
    assert b.to_list() == [1, 2, 2, 5, 5, 5, 9]
    assert b.count(5) == 3 and b.count(2) == 2 and b.count(3) == 0
    assert b.contains(2) and not b.contains(3)
    assert b.first == 1 and b.last == 9

    assert b.range(2..).first == 2 and
        b.range(3..).first == 5 and
        b.range(10..).first == None and
        b.range(10..).first == None
    assert b.range(..=5).last == 5 and
        b.range(..5).last == 2 and
        b.range(..=0).last == None and
        b.range(..1).last == None
    assert b.rank(1) == 0 and
        b.rank(2) == 1 and
        b.rank(3) == 3 and
        b.rank(5) == 3 and
        b.rank(6) == 6 and
        b.rank(10) == 7
    assert [b.nth(i)! for i in 0..b.len] == [1, 2, 2, 5, 5, 5, 9]
    assert b.nth(-1) == None and b.nth(7) == None

    // remove drops only one copy at a time
    assert b.remove(5) and b.count(5) == 2 and b.len == 6
    assert not b.remove(4) and b.len == 6
    assert b.pop_first() == 1 and b.pop_last() == 9
    assert b.to_list() == [2, 2, 5, 5]

    // take moves out in O(1) leaving empty; clear resets
    var moved = b.take()
    assert b.is_empty and b.len == 0 and b.first == None
    assert moved.to_list() == [2, 2, 5, 5]
    moved.clear()
    assert moved.is_empty and moved.len == 0 and moved.last == None

@test
fn sorted_bag_matches_sorted_list_on_random_ops():
    var b = SortedBag<Int>()
    var plain: List<Int> = []
    for _ in 0..400:
        let x = rand(0, 20)
        let op = rand(0, 3)
        if op == 0:
            assert b.add(x)
            var i = 0
            while i < plain.len and plain[i] <= x: i += 1
            plain.insert(i, x)
        elif op == 1:
            var i = 0
            while i < plain.len and plain[i] != x: i += 1
            assert b.remove(x) == (i < plain.len)
            if i < plain.len: plain = [plain[j] for j in 0..plain.len if j != i]
        elif op == 2:
            assert b.rank(x) == plain.count(v => v < x)
        else:
            let k = rand(-1, plain.len)
            assert b.nth(k) == (if 0 <= k < plain.len: Some(plain[k]) else: None)
        assert b.len == plain.len
        assert b.first == (if plain.len > 0: Some(plain[0]) else: None)
        assert b.last == (if plain.len > 0: Some(plain.last!) else: None)
    assert b.to_list() == plain

@test
fn sorted_bag_inversion_count_and_generic_keys():
    let a = [3, 1, 2, 3, 0, 2]
    var seen = SortedBag<Int>()
    var inv = 0
    for x in a:
        inv += seen.len - seen.rank(x + 1)
        let _ = seen.add(x)
    assert inv == a.pairs().count(pq => pq.0 > pq.1)
    var words = SortedBag<Str>()
    for w in ["pear", "apple", "fig", "apple"]: let _ = words.add(w)
    assert words.to_list() == ["apple", "apple", "fig", "pear"] and words.distinct == 3
    var pts = SortedBag<(Int, Int)>()
    for p in [(2, 1), (1, 5), (2, 0)]: let _ = pts.add(p)
    assert pts.first == Some((1, 5)) and pts.last == Some((2, 1)) and pts.rank((2, 0)) == 1

@test
fn sorted_bag_cursors_entries_and_recycling():
    var b = SortedBag<Int>()
    for x in [4, 1, 4, 7, 4]: let _ = b.add(x)
    assert b.entries().collect() == [(key: 1, count: 1), (key: 4, count: 3), (key: 7, count: 1)]
    assert b.entries_rev().collect() == [(key: 7, count: 1), (key: 4, count: 3), (key: 1, count: 1)]
    let c = b.cursor_at(4)!
    assert b.key_of(c) == 4 and b.count_of(c) == 3
    assert b.key_of(b.next_cursor(c)!) == 7 and b.key_of(b.prev_cursor(c)!) == 1
    assert b.next_cursor(b.cursor_last()!) == None and b.prev_cursor(b.cursor_first()!) == None
    assert b.key_of(b.cursor_ge(5)!) == 7 and b.key_of(b.cursor_gt(4)!) == 7
    assert b.key_of(b.cursor_le(3)!) == 1 and
        b.key_of(b.cursor_lt(4)!) == 1 and
        b.cursor_lt(1) == None
    for round in 0..50:
        for x in 0..20: let _ = b.add(x + round)
        assert b.len == 25 and b.distinct >= 20
        for x in 0..20: assert b.remove(x + round)
        assert b.to_list() == [1, 4, 4, 4, 7]

@test
fn sorted_set_operations_range_and_bsearch():
    var s = SortedSet<Int>()
    for x in [5, 1, 9, 5, 3]: let _ = s.add(x)
    assert s.to_list() == [1, 3, 5, 9] and s.len == 4
    assert not s.add(3) and s.remove(3) and not s.remove(3)
    s.set(4, true)
    s.set(9, false)
    assert s.to_list() == [1, 4, 5] and s.contains(4) and not s.contains(9)
    assert s.first_true(x => x >= 2) == Some(4) and s.first_true(x => x > 5) == None
    assert s.last_true(x => x < 5) == Some(4) and s.last_true(x => x < 1) == None
    assert s.rank(5) == 2 and s.nth(2) == Some(5) and s.nth(3) == None
    assert s.each().collect() == [1, 4, 5] and s.each_rev().collect() == [5, 4, 1]
    var t = SortedSet<Int>.of([4, 7])
    s.absorb(mut t)
    assert s.to_list() == [1, 4, 5, 7] and t.is_empty
    assert s.pop_first() == Some(1) and s.pop_last() == Some(7) and s.to_list() == [4, 5]

@test
fn sorted_set_and_bag_drain_after_before_and_random():
    var s = SortedSet<Int>.of([1, 3, 5, 7, 9, 12])
    assert s.drain(after: 3, while: x => x <= 9) == [5, 7, 9]
    assert [x for x in s] == [1, 3, 12]
    assert s.drain(before: 12, while: x => x >= 2) == [3]
    assert [x for x in s] == [1, 12]
    assert s.drain(while: x => x <= 5) == [1]
    assert [x for x in s] == [12]
    assert s.drain(after: 12, while: x => true) == []
    assert s.drain(before: 12, while: x => true) == []

    var b = SortedBag<Int>.of([2, 4, 4, 6, 8, 8, 10])
    assert b.drain(after: 2, while: x => x <= 6) == [4, 4, 6]
    assert b.drain(before: 10, while: x => x >= 8) == [8, 8]
    assert [x for x in b.range(..)] == [2, 10]

    // Randomized parity against a sorted list.
    var rs = SortedSet<Int>()
    var xs: List<Int> = []
    for _ in 0..80:
        let k = rand(0, 30)
        if rs.add(k):
            xs.push(k)
            xs.sort()
        if rand(0, 3) == 0:
            let pivot = rand(0, 30)
            let lim = rand(0, 30)
            if rand(0, 1) == 0:
                let got = rs.drain(after: pivot, while: x => x <= lim)
                var want: List<Int> = []
                var rest: List<Int> = []
                var draining = true
                for x in xs:
                    if x > pivot and draining and x <= lim:
                        want.push(x)
                    else:
                        if x > pivot: draining = false
                        rest.push(x)
                xs = rest
                assert got == want
            else:
                let got = rs.drain(before: pivot, while: x => x >= lim)
                var want: List<Int> = []
                var rest: List<Int> = []
                var draining = true
                for i in (0..xs.len).rev:
                    let x = xs[i]
                    if x < pivot and draining and x >= lim:
                        want.push(x)
                    else:
                        if x < pivot: draining = false
                        rest.push(x)
                rest.reverse()
                xs = rest
                assert got == want
            assert [x for x in rs] == xs

@test
fn sorted_bag_btree_grows_and_shrinks_against_a_count_table():
    let k = 1500
    var b = SortedBag<Int>()
    var cnt = [0 for _ in 0..k]
    for x in 0..k:
        b.add((x * 7) % k)
        cnt[(x * 7) % k] += 1
    check_bag_against_counts(b, cnt)
    for step in 0..3000:
        let x = rand(0, k - 1)
        if rand(0, 3) == 0:
            b.add_many(x, 2)
            cnt[x] += 2
        else:
            assert b.remove(x) == (cnt[x] > 0)
            if cnt[x] > 0: cnt[x] -= 1
        if step % 1000 == 999: check_bag_against_counts(b, cnt)
    for x in 0..k:
        assert b.remove_all(x) == cnt[x]
        cnt[x] = 0
        if x % 500 == 0: check_bag_against_counts(b, cnt)
    assert b.is_empty and b.cursor_first() == None and b.first == None
    for x in (0..k).rev:
        b.add(x)
        cnt[x] = 1
    check_bag_against_counts(b, cnt)
    let mid = b.split_at(k / 3)
    assert b.to_list() == (0..k / 3).collect() and mid.to_list() == (k / 3..k).collect()
    b.join(mid)
    check_bag_against_counts(b, cnt)
    let top = b.split_at(k - 10)
    assert top.to_list() == (k - 10..k).collect() and b.len == k - 10
    b.join(top)
    check_bag_against_counts(b, cnt)
fn check_bag_against_counts(b: SortedBag<Int>, cnt: List<Int>):
    // A bag over 0..k against a count table, through enough keys for three B+tree levels: node
    // splits while it grows, borrows and merges while it shrinks to empty, and every query near
    // the 16- and 32-slot node boundaries.
    var plain: List<Int> = []
    for x in 0..cnt.len:
        for _ in 0..cnt[x]: plain.push(x)
    assert b.len == plain.len and b.to_list() == plain
    assert b.distinct == cnt.count(c => c > 0)
    assert b.first == plain.first and b.last == plain.last
    var walked: List<Int> = []
    var c = b.cursor_first()
    while let at = c:
        walked.push(b.key_of(at))
        c = b.next_cursor(at)
    var back: List<Int> = []
    c = b.cursor_last()
    while let at = c:
        back.push(b.key_of(at))
        c = b.prev_cursor(at)
    back.reverse()
    assert walked == back and walked == [x for x in 0..cnt.len if cnt[x] > 0]
    for i in [
        0,
        1,
        15,
        16,
        17,
        31,
        32,
        33,
        47,
        48,
        63,
        64,
        65,
        plain.len / 2,
        plain.len - 1,
        plain.len,
    ]:
        assert b.nth(i) == (if 0 <= i < plain.len: Some(plain[i]) else: None)
    var below = 0
    for x in 0..cnt.len + 1:
        if x % 7 == 0 or x == cnt.len:
            assert b.rank(x) == below
            assert b.range(x..).first == plain.find(v => v >= x)
            assert b.range(..x).last == plain.rfind(v => v < x)
            assert b.first_true(v => v >= x) == plain.find(v => v >= x)
            assert b.last_true(v => v < x) == plain.rfind(v => v < x)
            assert b.range(x..x + 40).len == plain.count(v => x <= v < x + 40)
            for w in (if x % 49 == 0: [0, 1, 3, 700] else: []):
                let y = x + w
                assert b.range(x..=y).len == plain.count(v => x <= v <= y)
                assert b.range(y..x).len == plain.count(v => y <= v < x)
                assert b.range(x..y).len == plain.count(v => x <= v < y)
        if x < cnt.len:
            assert b.count(x) == cnt[x]
            below += cnt[x]

fn leaf_fills(b: SortedBag<Int>) -> List<Int>:
    var out: List<Int> = []
    var n = b.head
    while n != -1:
        out.push(b.fill(n))
        n = b.nxt(n)
    out

@test
fn first_and_last_true_find_every_cut_across_leaves():
    for m in [100, 300, 1000]:
        let b = SortedBag.of([i * 37 % 1009 for i in 0..m])
        let ks = b.to_list()
        for i in 0..ks.len:
            let k = ks[i]
            assert b.first_true(x => x >= k) == Some(k)
            assert b.last_true(x => x <= k) == Some(k)
            assert b.first_true(x => x > k) == (if i + 1 < ks.len: Some(ks[i + 1]) else: None)
            assert b.last_true(x => x < k) == (if i > 0: Some(ks[i - 1]) else: None)

@test
fn an_underfull_leaf_merges_into_a_sibling_that_fits_else_evens_out():
    for (more, gone, want) in [
        (33..41, 0..9, [32]),
        (-10..0, 16..26, [17, 16]),
        (33..42, 0..9, [16, 17]),
    ]:
        var b = SortedBag<Int>()
        for x in 0..33: b.add(x)
        assert leaf_fills(b) == [16, 17]
        for x in more: b.add(x)
        for x in gone: assert b.remove(x)
        assert leaf_fills(b) == want
        var plain = (0..33).collect() + more.collect()
        plain = [x for x in plain.sorted() if not gone.contains(x)]
        var back: List<Int> = []
        var c = b.cursor_last()
        while let at = c:
            back.push(b.key_of(at))
            c = b.prev_cursor(at)
        assert b.to_list() == plain and back == plain.rev

@test
fn sorted_map_btree_set_remove_and_walk_against_a_table():
    let k = 1200
    var m: SortedMap<Int, Str> = SortedMap()
    var want: List<?Str> = [None for _ in 0..k]
    for step in 0..5000:
        let x = rand(0, k - 1)
        if rand(0, 2) > 0:
            m.set(x, "v{step}")
            want[x] = Some("v{step}")
        else:
            assert m.remove(x) == want[x]
            want[x] = None
        if step % 1000 == 999:
            let have = [(x, want[x]!) for x in 0..k if want[x] != None]
            assert m.items.collect() == have and m.len == have.len
            assert m.each_rev().collect() == have.reversed()
            assert m.keys.collect() == [e.0 for e in have] and
                m.values.collect() == [e.1 for e in have]
            for i in [0, 15, 16, 31, 32, 33, have.len - 1]:
                assert m.nth(i) == (if 0 <= i < have.len: Some(have[i]) else: None)
            for y in 0..k:
                if y % 37 == 0:
                    assert m.get(y) == want[y] and m.contains(y) == (want[y] != None)
                    assert m.first_ge(y) == have.find(e => e.0 >= y)
                    assert m.last_lt(y) == have.rfind(e => e.0 < y)
                    assert m.first_true((key, _) => key >= y) == have.find(e => e.0 >= y)
                    assert m.rank(y) == have.count(e => e.0 < y)
    let n = m.len
    let hi = m.split_at(k / 2)
    assert m.len + hi.len == n and (m.last == None or m.last!.0 < k / 2)
    m.join(hi)
    assert m.len == n
    let drained = m.drain(after: 100, while: (key, _) => key < 400)
    assert drained == [(x, want[x]!) for x in 101..400 if want[x] != None]
    assert m.len == n - drained.len

/// The keys of a `SortedBag` or `SortedSet` inside a range, made by `range(r)`. A view: it stores
/// the bounds and lends the container, and allocates nothing. `for x in s.range(r)` iterates
/// `each()`. Copies of one key in a `SortedBag` are all inside or all outside, so nothing here
/// has a tie to break. `n` is the container's `len`, `k` the number of keys visited.
pub struct SortedRange<K>:
    bag: SortedBag<K>
    lo: ?K
    hi: ?K
    hi_inc: Bool

    /// The smallest key in the range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).first == None`
    /// ties: `let b = SortedBag.of([1, 3, 3]); b.range(2..5).first == Some(3) and b.range(3..=3).first == Some(3) and b.range(..=1).first == Some(1) and b.range(..1).first == None`
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(2..).first == Some(3)`
    @get pub fn first(self) -> ?K = bag.range_first(lo, hi, hi_inc)

    /// The largest key in the range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).last == None`
    /// ties: `let b = SortedBag.of([3, 3, 5]); b.range(2..5).last == Some(3) and b.range(3..4).last == Some(3) and b.range(5..).last == Some(5) and b.range(..=3).last == Some(3)`
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..=6).last == Some(3)`
    @get pub fn last(self) -> ?K = bag.range_last(lo, hi, hi_inc)

    /// The number of keys in the range, counting copies.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).len == 0`
    /// ties: `let b = SortedBag.of([3, 3, 5, 5]); b.range(3..=5).len == 4 and b.range(3..5).len == 2 and b.range(..=3).len == 2 and b.range(5..).len == 2 and b.range(..5).len == 2`
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(3..=3).len == 2`
    @get pub fn len(self) -> Int = bag.range_len(lo, hi, hi_inc)

    /// True when the range holds no key.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).is_empty == true`
    /// traps: never
    /// example: `SortedSet.of([1, 7]).range(2..7).is_empty`
    @get pub fn is_empty(self) -> Bool = self.first == None

    /// The keys in ascending order, copies included.
    ///
    /// cost: \(O(\log n + k)\) expected for `k` keys visited
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).each().collect() == []`
    /// order: ascending
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(2..).each().collect() == [3, 3, 7]`
    pub fn each(self) -> yields K = bag.range_each(lo, hi, hi_inc)

    /// The keys in descending order, copies included.
    ///
    /// cost: \(O(\log n + k)\) expected for `k` keys visited
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).rev.collect() == []`
    /// order: descending
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..7).rev.collect() == [3, 1]`
    /// example: `let b = SortedBag.of([1, 3, 3, 7]); [x for x in b.range(2..).rev] == [7, 3, 3]`
    @get pub fn rev(self) -> yields K = bag.range_each_rev(lo, hi, hi_inc)

/// `push(x)` adds one copy of `x` and `pop(x)` removes one.
impl Track<K> for SortedBag<K>:
    fn push(mut self, x: K):
        self.add(x)
    fn pop(mut self, x: K):
        self.remove(x)

@test
fn bag_family_walks_and_shows():
    var m: SortedMap<Int, Str> = SortedMap()
    m.set(3, "c")
    m.set(1, "a")
    assert "{m}" == "1=a, 3=c"
    var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k)
    for x in [3, 1, 3]: b.add(x)
    assert "{b}" == "1 3 3"
    var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k)
    for x in [3, 1, 3]: s.add(x)
    assert "{s}" == "1 3"
    var c = BucketCount(0..30)
    for x in [29, 0, 7, 7, 15]: c.add(x)
    assert c.each().collect() == [0, 7, 7, 15, 29] and "{c}" == "0 7 7 15 29"
    c.remove(7, 2)
    assert c.each().collect() == [0, 15, 29]
    assert "{SortedBag.of([1, 3, 3, 7]).range(2..)}" == "3 3 7"

impl Show for SortedBag<K>:
    fn show(b):
        var out = ""
        for x in b.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for SortedSet<K>:
    fn show(s):
        var out = ""
        for x in s.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for SortedMap<K, V>:
    fn show(v):
        var out = ""
        for (k, x) in v.items:
            if not out.is_empty: out += ", "
            out += "{k}={x}"
        out

impl Show for AugSortedBag<K, T>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for AugSortedSet<K, T>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for BucketCount:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Seq<K> for SortedRange<K>:
    fn iter(self) -> yields K:
        for x in self.each(): yield x

impl Show for SortedRange<K>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for BagCursor:
    /// The slot it points at, as `cursor at slot i`; read its key with `key_of`.
    ///
    /// example: `let b = SortedBag.of([5, 2, 2]); "{b.cursor_last()!}" == "cursor at slot 1"`
    fn show(c) = "cursor at slot {c.id}"

// One copy for every key type: the weights are `Int` whatever `K` is.
fn slot_sum(ws: List<Int>, a: Int, f: Int) -> Int = sum(ws[a + i] for i in 0..f)

const AFF_MOD = 1_000_003

fn aff(f: (Int, Int), g: (Int, Int)) -> (Int, Int) = (
    f.0 * g.0 % AFF_MOD,
    (f.1 * g.0 + g.1) % AFF_MOD,
)

@test
fn aug_sorted_bag_folds_stay_right_across_splits_and_merges():
    let k = 1200
    var b = AugSortedBag<Int, (Int, Int)>((f, g) => aff(f, g), (1, 0), x => (x % 7 + 2, x))
    var cnt = [0 for _ in 0..k]
    for x in 0..k:
        b.add(x * 7 % k)
        cnt[x * 7 % k] += 1
    check_aug(b, cnt)
    for step in 0..4000:
        let x = rand(0, k - 1)
        if rand(0, 2) == 0:
            b.add(x)
            cnt[x] += 1
        else:
            assert b.remove(x) == (cnt[x] > 0)
            if cnt[x] > 0: cnt[x] -= 1
        if step % 800 == 799: check_aug(b, cnt)
    for x in 0..k:
        while cnt[x] > 0:
            assert b.remove(x)
            cnt[x] -= 1
        if x % 300 == 0: check_aug(b, cnt)
    assert b.is_empty and b.all == (1, 0) and b.first == None
fn aff_fold(have: List<Int>, ok: Int -> Bool) -> (Int, Int):
    var acc = (1, 0)
    for x in have:
        if ok(x): acc = aff(acc, (x % 7 + 2, x))
    acc

fn check_aug(b: AugSortedBag<Int, (Int, Int)>, cnt: List<Int>):
    var have: List<Int> = []
    for x in 0..cnt.len:
        for _ in 0..cnt[x]: have.push(x)
    assert b.len == have.len and b.each().collect() == have
    assert b.all == aff_fold(have, _ => true)
    if have.len > 0: assert b.first == Some(have[0]) and b.last == have.last
    for _ in 0..12:
        let lo = rand(0, cnt.len)
        let hi = rand(0, cnt.len)
        assert b.fold_lt(lo) == aff_fold(have, x => x < lo)
        assert b.fold_le(lo) == aff_fold(have, x => x <= lo)
        assert b.fold_ge(lo) == aff_fold(have, x => x >= lo)
        assert b.fold_gt(lo) == aff_fold(have, x => x > lo)
        assert b.fold_between(lo, hi) == aff_fold(have, x => lo <= x < hi)
        assert b.count(lo) == (if lo < cnt.len: cnt[lo] else: 0)
        assert b.rank(lo) == have.count(x => x < lo)
        if lo < have.len: assert b.nth(lo) == Some(have[lo])

// Which bag keeps its nodes when two meet with equal sizes: `self`, as the docs of `split_at`,
// `join` and `absorb` say.
@test
fn sorted_bag_ties_keep_selfs_node_array():
    var s = SortedBag<Int>.new()
    for i in 0..200: s.add(i)
    let m = s.meta.len
    let hi = s.split_at(100)
    assert s.meta.len == m and hi.meta.len < m and s.len == 100 and hi.len == 100
    for joined in [false, true]:
        var a = SortedBag<Int>.new()
        for i in 0..300: a.add(i)
        for i in 100..300: let _ = a.remove(i)
        let ma = a.meta.len
        var o = SortedBag<Int>.new()
        for i in 100..200: o.add(i)
        if joined: a.join(o)
        else: a.absorb(mut o)
        assert a.meta.len == ma and a.to_list() == (0..200).collect()

// A bag holds only the room its nodes use: the first node grows slot by slot, a leaf has no
// child slots, every later node takes a whole node's slots, and a reused node keeps its child
// slots.
@test
fn sorted_bag_holds_only_the_room_its_nodes_use():
    var c = SortedBag<Int>.new()
    for i in 0..3: c.add(i)
    assert c.keys.len == 3 and c.ws.len == 3 and c.kids.len == 0
    var d = SortedBag<Int>.new()
    for i in 0..200: d.add(i)
    assert d.keys.len == d.meta.len / 4 * BAG_B and d.kids.len == BAG_B
    var e = SortedBag.of((0..8).collect())
    for _ in 0..5:
        for i in 8..64: e.add(i)
        assert e.kids.len <= 2 * BAG_B
        for i in 8..64: let _ = e.remove(i)
        assert e.height == 0 and e.to_list() == (0..8).collect()
