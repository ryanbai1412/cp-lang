//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`.

use std.seq.{remove}

/// Read-only handle into a `SortedBag<K>` node, valid until the next mutation.
pub struct BagCursor(id: Int)

struct BagNode<K>:
    key: K
    var cnt: Int
    var sub: Int
    pri: -2_147_483_648..=2_147_483_647
    var left: -2_147_483_648..=2_147_483_647
    var right: -2_147_483_648..=2_147_483_647
    var par: -2_147_483_648..=2_147_483_647

/// An ordered multiset (bag) of `K` backed by a randomized treap over a single flat node array
/// with recycled node slots and parent pointers for \(O(1)\) amortized cursor stepping.
/// Keeps one node per distinct key with its multiplicity and subtree element count.
/// `add`, `add_many`, `remove`, `remove_all`, `contains`, `count`,
/// `pop_first`, `pop_last`, `rank`, `nth`, `first_true`, `last_true`, and `cursor_*` are \(O(\log n)\) expected;
/// `range(r)` returns an \(O(1)\) view whose `.first`, `.last`, `.len`, `.is_empty` are \(O(\log n)\)
/// expected and whose iteration (`for x in b.range(r)` / `.rev`) is \(O(\log n + k)\);
/// `first`, `last`, `key_of` and `count_of` are \(O(1)\); `next_cursor` and `prev_cursor` are \(O(1)\) amortized;
/// `len`, `distinct`, `is_empty`, `clear`, and `take` are \(O(1)\);
/// `entries`, `entries_rev`, and `to_list` are \(O(n)\).
///
/// Elements are ordered by `K`'s `<`; copies of one key are equal and indistinguishable, so
/// no method here has a tie to break between them. `n` in every cost is `len`.
pub struct SortedBag<K>:
    var nodes: List<BagNode<K>>
    var free: List<-2_147_483_648..=2_147_483_647>
    var live: Int
    var root: Int
    var rng: U64
    var lo: ?K
    var hi: ?K

    /// Create an empty `SortedBag<K>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag(); b.len == 0`
    pub fn new() -> SortedBag<K> = SortedBag([], [], 0, -1, U64(0x5e3779b97f4a7c15), None, None)

    /// Build a `SortedBag` containing every element of `xs`.
    ///
    /// Every copy counts: `SortedBag.of([2, 2])` holds two elements.
    ///
    /// cost: \(O(n \log n)\) expected
    /// empty: an empty bag
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).to_list() == [1, 3, 3]`
    pub fn of(xs: List<K>) -> SortedBag<K>:
        var b = SortedBag<K>.new()
        for x in xs: b.add(x)
        b

    /// Total number of elements in the bag (counting duplicates).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).len == 3`
    @get pub fn len(self) -> Int:
        self.sub_of(root)

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
        nodes = []
        free = []
        live = 0
        root = -1
        lo = None
        hi = None

    /// Move the bag's contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); let t = b.take(); b.is_empty and t.to_list() == [1, 2]`
    pub fn take(mut self) -> SortedBag<K>:
        let out = SortedBag(nodes, free, live, root, rng, lo, hi)
        nodes = []
        free = []
        live = 0
        root = -1
        lo = None
        hi = None
        out

    /// Insert one copy of `x` into the bag. Always returns `true`.
    ///
    /// The `Bool` matches `SortedSet.add`, so code can switch between the two.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag(); b.add(3) and b.add(3) and b.count(3) == 2`
    pub fn add(mut self, x: K) -> Bool:
        let (r, _) = self.insert_at(root, x, 1, false)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        self.refresh()
        true

    fn add_unique(mut self, x: K) -> Bool:
        let (r, added) = self.insert_at(root, x, 1, true)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        self.refresh()
        added

    /// Insert `c >= 0` copies of `x` into the bag.
    ///
    /// `c == 0` does nothing.
    ///
    /// cost: \(O(\log n)\) expected, independent of `c`
    /// traps: when `c < 0`
    /// example: `var b = SortedBag.of([1]); b.add_many(5, 3); b.len == 4 and b.count(5) == 3`
    pub fn add_many(mut self, x: K, c: Int):
        assert c >= 0
        if c == 0: return
        let (r, _) = self.insert_at(root, x, c, false)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        self.refresh()

    /// Remove one copy of `x` if present; returns `true` if a copy was removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2]); b.remove(2) and b.count(2) == 1 and not b.remove(7)`
    pub fn remove(mut self, x: K) -> Bool:
        let (r, removed) = self.erase_at(root, x, false)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        self.refresh()
        removed > 0

    /// Remove all copies of `x` and return how many were removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2, 3]); b.remove_all(2) == 2 and b.to_list() == [3]`
    /// example: `var b = SortedBag.of([3]); b.remove_all(2) == 0`
    pub fn remove_all(mut self, x: K) -> Int:
        let (r, removed) = self.erase_at(root, x, true)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        self.refresh()
        removed

    /// How many copies of `x` are currently in the bag.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `SortedBag.of([2, 2, 3]).count(2) == 2`
    /// example: `SortedBag.of([2, 2, 3]).count(4) == 0`
    pub fn count(self, x: K) -> Int:
        var u = root
        unchecked:
            while u >= 0:
                if x < nodes[u].key: u = nodes[u].left
                elif nodes[u].key < x: u = nodes[u].right
                else: return nodes[u].cnt
        0

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
    /// empty: `None`
    /// ties: none: copies of the least key are equal
    /// traps: never
    /// example: `SortedBag.of([3, 1, 1]).first == Some(1)`
    @get pub fn first(self) -> ?K:
        lo

    /// The largest element, or `None` when empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: copies of the greatest key are equal
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).last == Some(3)`
    @get pub fn last(self) -> ?K:
        hi

    /// Remove and return one copy of the smallest element, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var b = SortedBag.of([3, 1, 1]); b.pop_first() == Some(1) and b.count(1) == 1`
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
    /// empty: `None`
    /// ties: none: copies of the answer are equal; one is removed
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `var b = SortedBag.of([1, 4, 4, 6]); b.take_first(x => x >= 3) == Some(4) and b.to_list() == [1, 4, 6]`
    /// example: `var b = SortedBag.of([1]); b.take_first(x => x >= 3) == None and b.len == 1`
    pub fn take_first(mut self, ok: K -> Bool) -> ?K:
        let x = self.first_true(ok)
        if let v = x: let _ = self.remove(v)
        x

    /// \(O(\log n + k)\). Removes the elements whose value lies in `r`, yielding them in order.
    ///
    /// Each copy is yielded once, so a key with count `c` is yielded `c` times. A loop that
    /// stops early leaves the elements not yet yielded in the bag. `r` may be any ordered range
    /// (`a..b`, `a..=b`, `..b`, `a..`, `..`).
    ///
    /// cost: \(O(\log n + k)\) expected for `k` removed
    /// empty: yields nothing, for an empty bag or an empty `r`
    /// order: ascending
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5, 7]); b.drain_range(3..6).collect() == [3, 3, 5] and b.to_list() == [1, 7]`
    pub fn drain_range(mut self, r: Range<K>) -> yields K:
        if r.is_empty: return
        while let x = self.range_first(Some(r.l), true, Some(r.r), false):
            let _ = self.remove(x)
            yield x

    /// Remove and return one copy of the largest element, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var b = SortedBag.of([3, 3, 1]); b.pop_last() == Some(3) and b.to_list() == [1, 3]`
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
    /// empty: `0`
    /// traps: never
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(2) == 1`
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(3) == 3`
    pub fn rank(self, k: K) -> Int:
        var ans = 0
        var u = root
        unchecked:
            while u >= 0:
                if nodes[u].key < k:
                    let l = nodes[u].left
                    ans += (if l < 0: 0 else: nodes[l].sub) + nodes[u].cnt
                    u = nodes[u].right
                else:
                    u = nodes[u].left
        ans

    fn rank_le(self, k: K) -> Int:
        var ans = 0
        var u = root
        unchecked:
            while u >= 0:
                if not (k < nodes[u].key):
                    let l = nodes[u].left
                    ans += (if l < 0: 0 else: nodes[l].sub) + nodes[u].cnt
                    u = nodes[u].right
                else:
                    u = nodes[u].left
        ans

    fn lo_ok(self, x: K, lo: ?K, lo_inc: Bool) -> Bool:
        if let a = lo:
            if lo_inc: not (x < a) else: a < x
        else:
            true

    fn hi_ok(self, x: K, hi: ?K, hi_inc: Bool) -> Bool:
        if let b = hi:
            if hi_inc: not (b < x) else: x < b
        else:
            true

    /// The keys of `self` inside `r`, as an \(O(1)\) view; replaces `first_ge`, `first_gt`, `last_le`,
    /// `last_lt` and `count_range` (decision 103). `r` is any ordered range: `a..b`, `a..=b`, `..b`,
    /// `..=b`, `a..` or `..`. The view lends `self`, so `self` cannot change while it is in use.
    ///
    /// cost: \(O(1)\) to make; see `SortedRange` for each query
    /// empty: a range with no keys gives an empty view
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(2..=7).len == 3`
    pub fn range(self, r: Range<K>) -> SortedRange<K> = todo

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K:
        var ans = -1
        unchecked:
            if let a = lo:
                var u = root
                if lo_inc:
                    while u >= 0:
                        if not (nodes[u].key < a):
                            ans = u
                            u = nodes[u].left
                        else:
                            u = nodes[u].right
                else:
                    while u >= 0:
                        if a < nodes[u].key:
                            ans = u
                            u = nodes[u].left
                        else:
                            u = nodes[u].right
            else:
                var u = root
                while u >= 0:
                    ans = u
                    u = nodes[u].left
            if ans < 0: return None
            let k = nodes[ans].key
            return if self.hi_ok(k, hi, hi_inc): k else: None

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K:
        var ans = -1
        unchecked:
            if let b = hi:
                var u = root
                if hi_inc:
                    while u >= 0:
                        if not (b < nodes[u].key):
                            ans = u
                            u = nodes[u].right
                        else:
                            u = nodes[u].left
                else:
                    while u >= 0:
                        if nodes[u].key < b:
                            ans = u
                            u = nodes[u].right
                        else:
                            u = nodes[u].left
            else:
                var u = root
                while u >= 0:
                    ans = u
                    u = nodes[u].right
            if ans < 0: return None
            let k = nodes[ans].key
            return if self.lo_ok(k, lo, lo_inc): k else: None

    fn range_len(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> Int:
        let cnt_hi = if let b = hi:
            if hi_inc: self.rank_le(b) else: self.rank(b)
        else:
            self.len
        let cnt_lo = if let a = lo:
            if lo_inc: self.rank(a) else: self.rank_le(a)
        else:
            0
        if cnt_hi > cnt_lo: cnt_hi - cnt_lo else: 0

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        var cur = if let a = lo:
            if lo_inc: self.cursor_ge(a) else: self.cursor_gt(a)
        else:
            self.cursor_first()
        unchecked:
            while let c = cur:
                let k = nodes[c.id].key
                if not self.hi_ok(k, hi, hi_inc): break
                for _ in 0..nodes[c.id].cnt: yield k
                cur = self.next_cursor(c)

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        var cur = if let b = hi:
            if hi_inc: self.cursor_le(b) else: self.cursor_lt(b)
        else:
            self.cursor_last()
        unchecked:
            while let c = cur:
                let k = nodes[c.id].key
                if not self.lo_ok(k, lo, lo_inc): break
                for _ in 0..nodes[c.id].cnt: yield k
                cur = self.prev_cursor(c)

    /// 0-indexed `i`-th smallest element (counting duplicates), or `None` if out of bounds.
    ///
    /// It is `b.to_list()[i]` without the list; `None` for any `i` outside `0..len`, negative
    /// included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2, 1]).nth(2) == Some(2)`
    /// example: `SortedBag.of([5, 2, 2, 1]).nth(4) == None`
    pub fn nth(self, i: Int) -> ?K:
        if i < 0 or i >= self.len: return None
        var k = i
        var u = root
        unchecked:
            while u >= 0:
                let l = nodes[u].left
                let lsz = if l < 0: 0 else: nodes[l].sub
                if k < lsz:
                    u = l
                elif k < lsz + nodes[u].cnt:
                    return nodes[u].key
                else:
                    k -= lsz + nodes[u].cnt
                    u = nodes[u].right
        None

    /// Smallest key where monotone `ok(k)` holds (`false..false, true..true`), or `None`.
    /// Checks monotonicity at the boundary. \(O(\log n)\) expected.
    ///
    /// `ok` is called on keys along one root-to-leaf path. Debug builds check that `ok` is
    /// false on the key just below the answer.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `None`
    /// ties: none: copies of a key are one candidate; the least key wins
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x > 9) == None`
    pub fn first_true(self, ok: K -> Bool) -> ?K:
        var best: ?K = None
        var u = root
        unchecked:
            while u >= 0:
                if ok(nodes[u].key):
                    best = Some(nodes[u].key)
                    u = nodes[u].left
                else:
                    u = nodes[u].right
        best

    /// Largest key where monotone `ok(k)` holds (`true..true, false..false`), or `None`.
    /// Checks monotonicity at the boundary. \(O(\log n)\) expected.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `None`
    /// ties: none: copies of a key are one candidate; the greatest key wins
    /// traps: never; an `ok` that is not true-then-false gives an unspecified answer
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x < 1) == None`
    pub fn last_true(self, ok: K -> Bool) -> ?K:
        var best: ?K = None
        var u = root
        unchecked:
            while u >= 0:
                if ok(nodes[u].key):
                    best = Some(nodes[u].key)
                    u = nodes[u].right
                else:
                    u = nodes[u].left
        best

    /// Remove and return the maximal run of elements strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest element (ascending)
    /// while `ok(k)` holds. \(O(k \log n)\) expected for `k` removed elements.
    ///
    /// The walk starts at the first element past the bound and stops at the first element
    /// where `ok` is false, which stays; each copy of a removed key is one element of the
    /// result. Giving both `after` and `before` is an error.
    ///
    /// cost: \(O(k \log n)\) expected for `k` removed elements
    /// empty: `[]`
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
            while let x = self.range_last(None, false, Some(b), false):
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        elif let a = after:
            while let x = self.range_first(Some(a), false, None, false):
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
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_first()!) == 2`
    pub fn cursor_first(self) -> ?BagCursor:
        if root < 0: return None
        var u = root
        unchecked:
            while nodes[u].left >= 0: u = nodes[u].left
        BagCursor(u)

    /// Cursor to the largest key, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_last()!) == 5`
    pub fn cursor_last(self) -> ?BagCursor:
        if root < 0: return None
        var u = root
        unchecked:
            while nodes[u].right >= 0: u = nodes[u].right
        BagCursor(u)

    /// Cursor to exact key `k`, or `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.cursor_at(5) != None and b.cursor_at(3) == None`
    pub fn cursor_at(self, k: K) -> ?BagCursor:
        var u = root
        unchecked:
            while u >= 0:
                if k < nodes[u].key: u = nodes[u].left
                elif nodes[u].key < k: u = nodes[u].right
                else: return BagCursor(u)
        None

    /// Cursor to smallest key `>= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_ge(3)!) == 5 and b.cursor_ge(6) == None`
    pub fn cursor_ge(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        unchecked:
            while u >= 0:
                if not (nodes[u].key < k):
                    ans = u
                    u = nodes[u].left
                else:
                    u = nodes[u].right
        if ans >= 0: BagCursor(ans) else: None

    /// Cursor to smallest key `> k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_gt(2)!) == 5 and b.cursor_gt(5) == None`
    pub fn cursor_gt(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        unchecked:
            while u >= 0:
                if k < nodes[u].key:
                    ans = u
                    u = nodes[u].left
                else:
                    u = nodes[u].right
        if ans >= 0: BagCursor(ans) else: None

    /// Cursor to largest key `<= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_le(4)!) == 2 and b.cursor_le(1) == None`
    pub fn cursor_le(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        unchecked:
            while u >= 0:
                if not (k < nodes[u].key):
                    ans = u
                    u = nodes[u].right
                else:
                    u = nodes[u].left
        if ans >= 0: BagCursor(ans) else: None

    /// Cursor to largest key `< k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_lt(5)!) == 2 and b.cursor_lt(2) == None`
    pub fn cursor_lt(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        unchecked:
            while u >= 0:
                if nodes[u].key < k:
                    ans = u
                    u = nodes[u].right
                else:
                    u = nodes[u].left
        if ans >= 0: BagCursor(ans) else: None

    /// Key stored at cursor `c`.
    ///
    /// `c` must come from this bag and no mutation may have happened since it was made.
    ///
    /// cost: \(O(1)\)
    /// traps: never in release (a stale cursor reads an unspecified key); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.cursor_first()!) == 2`
    pub fn key_of(self, c: BagCursor) -> K = nodes[c.id].key

    /// Multiplicity stored at cursor `c`.
    ///
    /// cost: \(O(1)\)
    /// traps: never in release (a stale cursor reads an unspecified count); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.count_of(b.cursor_at(2)!) == 2`
    pub fn count_of(self, c: BagCursor) -> Int = nodes[c.id].cnt

    /// Cursor to the next larger distinct key, or `None`.
    ///
    /// Steps over all copies of the current key at once.
    ///
    /// cost: \(O(1)\) amortized, \(O(\log n)\) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.next_cursor(b.cursor_first()!)!) == 5`
    /// example: `let b = SortedBag.of([2, 5]); b.next_cursor(b.cursor_last()!) == None`
    pub fn next_cursor(self, c: BagCursor) -> ?BagCursor:
        var u = c.id
        unchecked:
            if nodes[u].right >= 0:
                u = nodes[u].right
                while nodes[u].left >= 0: u = nodes[u].left
                return BagCursor(u)
            while nodes[u].par >= 0:
                let p = nodes[u].par
                if nodes[p].left == u: return BagCursor(p)
                u = p
        None

    /// Cursor to the next smaller distinct key, or `None`.
    ///
    /// cost: \(O(1)\) amortized, \(O(\log n)\) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 5, 5]); b.key_of(b.prev_cursor(b.cursor_last()!)!) == 2`
    /// example: `let b = SortedBag.of([2, 5]); b.prev_cursor(b.cursor_first()!) == None`
    pub fn prev_cursor(self, c: BagCursor) -> ?BagCursor:
        var u = c.id
        unchecked:
            if nodes[u].left >= 0:
                u = nodes[u].left
                while nodes[u].right >= 0: u = nodes[u].right
                return BagCursor(u)
            while nodes[u].par >= 0:
                let p = nodes[u].par
                if nodes[p].right == u: return BagCursor(p)
                u = p
        None

    /// Visit every distinct `(key, count)` in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries()] == [2, 5]`
    /// example: `[e.count for e in SortedBag.of([5, 2, 2]).entries()] == [2, 1]`
    pub fn entries(self) -> yields (key: K, count: Int):
        var cur = self.cursor_first()
        while let c = cur:
            yield (key: nodes[c.id].key, count: nodes[c.id].cnt)
            cur = self.next_cursor(c)

    /// Visit every distinct `(key, count)` in descending order.
    ///
    /// Exactly the reverse of `entries`.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: descending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries_rev()] == [5, 2]`
    pub fn entries_rev(self) -> yields (key: K, count: Int):
        var cur = self.cursor_last()
        while let c = cur:
            yield (key: nodes[c.id].key, count: nodes[c.id].cnt)
            cur = self.prev_cursor(c)

    /// Visit every element, duplicates included, in ascending order; `for x in b` uses this.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending, each key `count` times
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2]).each().collect() == [2, 2, 5]`
    pub fn each(self) -> yields K:
        for k in self.range_each(None, false, None, false): yield k

    /// Visit every element, duplicates included, in descending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: descending, each key `count` times
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2]).each_rev().collect() == [5, 2, 2]`
    pub fn each_rev(self) -> yields K:
        for k in self.range_each_rev(None, false, None, false): yield k

    /// Every element in ascending order, repeated by multiplicity.
    ///
    /// cost: \(O(n)\)
    /// empty: `[]`
    /// order: ascending, each key repeated by its count; a new `List`, keyed from 0
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).to_list() == [1, 3, 3]`
    pub fn to_list(self) -> List<K>:
        var out: List<K> = []
        for (k, c) in self.entries():
            for _ in 0..c: out.push(k)
        out

    /// Preallocate treap node pool for at least `cap` distinct keys.
    ///
    /// cost: \(O(\mathit{cap})\) when the pool grows, \(O(1)\) otherwise
    /// traps: never
    /// example: `var b = SortedBag.of([1]); b.reserve(100); b.to_list() == [1]`
    pub fn reserve(mut self, cap: Int):
        if cap < 0: return

    /// Split off and return all elements `>= k`, keeping `< k` in `self`.
    ///
    /// Every copy of a key goes to the same side.
    ///
    /// The larger side keeps the node array; the smaller side's keys move to a fresh one.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's distinct keys
    /// empty: both sides empty
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5]); let hi = b.split_at(3); b.to_list() == [1] and hi.to_list() == [3, 3, 5]`
    pub fn split_at(mut self, k: K) -> SortedBag<K>:
        let (l, r) = self.split_lt(root, k)
        unchecked:
            if l >= 0: nodes[l].par = -1
            if r >= 0: nodes[r].par = -1
        var out = SortedBag<K>.new()
        if self.sub_of(l) < self.sub_of(r):
            out = self.take()
            out.root = r
            out.move_out(l, mut self)
        else:
            root = l
            self.move_out(r, mut out)
        self.refresh()
        out.refresh()
        out

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The inverse of `split_at`. The precondition is checked in \(O(1)\) against `self.last` and
    /// `other.first`; either side may be empty.
    ///
    /// The smaller side's keys move into the larger side's node array, as in `split_at`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's distinct keys
    /// traps: when both are non-empty and `self.last >= other.first`
    /// example: `var a = SortedBag.of([1, 2]); a.join(SortedBag.of([4, 4])); a.to_list() == [1, 2, 4, 4]`
    pub fn join(mut self, other: SortedBag<K>):
        if let a = hi:
            if let b = other.lo: assert a < b, "`join` needs every key of `self` below every key of `other`"
        var o = other
        if self.sub_of(root) >= o.sub_of(o.root):
            o.move_out(o.root, mut self)
        else:
            self.move_out(root, mut o)
            nodes = o.nodes
            free = o.free
            live = o.live
            root = o.root
            rng = o.rng
        self.refresh()

    /// Merge all elements of `other` into `self` (small-to-large, emptying `other`).
    ///
    /// Copies add up: a key with count `a` in `self` and `b` in `other` ends with `a + b`. The
    /// smaller bag's elements are inserted into the larger, whichever side that is.
    ///
    /// cost: \(O(\min(n, m) \log(n + m))\) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a = SortedBag.of([1, 3]); var b = SortedBag.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedBag<K>):
        if self.len < other.len:
            let (n0, f0, l0, r0, g0, lo0, hi0) = (nodes, free, live, root, rng, lo, hi)
            nodes = other.nodes
            free = other.free
            live = other.live
            root = other.root
            rng = other.rng
            lo = other.lo
            hi = other.hi
            other.nodes = n0
            other.free = f0
            other.live = l0
            other.root = r0
            other.rng = g0
            other.lo = lo0
            other.hi = hi0
        for (k, c) in other.entries(): self.add_many(k, c)
        other.clear()

    /// Sum of all elements (for additive `K`). \(O(1)\) when augmented, \(O(n)\) otherwise.
    ///
    /// Each copy is added. `BagNode` keeps no subtree sum, so this bag is never augmented and
    /// the cost is \(O(n)\); `AugSortedBag` keeps sums.
    ///
    /// cost: \(O(n)\)
    /// empty: `K.zero`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `SortedBag.of([2, 2, 5]).sum() == 9`
    pub fn sum(self) -> K:
        var t = K.zero
        for (k, c) in self.entries():
            for _ in 0..c: t = t + k
        t

    fn refresh(mut self):
        if root < 0:
            lo = None
            hi = None
            return
        unchecked:
            var u = root
            while nodes[u].left >= 0: u = nodes[u].left
            lo = Some(nodes[u].key)
            u = root
            while nodes[u].right >= 0: u = nodes[u].right
            hi = Some(nodes[u].key)

    fn sub_of(self, t: Int) -> Int:
        if t < 0: return 0
        unchecked: return nodes[t].sub

    fn pull(mut self, u: Int):
        unchecked:
            let l = nodes[u].left
            let r = nodes[u].right
            nodes[u].sub = nodes[u].cnt + (if l < 0: 0 else: nodes[l].sub) + (
                if r < 0: 0 else: nodes[r].sub
            )
            if l >= 0: nodes[l].par = u
            if r >= 0: nodes[r].par = u

    fn make_node(mut self, x: K, c: Int) -> Int:
        live += 1
        rng = rng * U64(6364136223846793005) + U64(1442695040888963407)
        let p = Int((rng >> 33) & U64(0x7fffffff))
        unchecked:
            if let u = free.pop():
                nodes[u] = BagNode(x, c, c, p, -1, -1, -1)
                return u
            nodes.push(BagNode(x, c, c, p, -1, -1, -1))
            return nodes.len - 1

    fn insert_at(mut self, u: Int, x: K, c: Int, unique: Bool) -> (Int, Bool):
        if u < 0:
            return (self.make_node(x, c), true)
        unchecked:
            if x < nodes[u].key:
                let old_l = nodes[u].left
                let (nl, added) = self.insert_at(old_l, x, c, unique)
                if not added: return (u, false)
                if nl != old_l:
                    nodes[u].left = nl
                    if nodes[nl].pri > nodes[u].pri:
                        let lr = nodes[nl].right
                        nodes[u].left = lr
                        nodes[nl].right = u
                        self.pull(u)
                        self.pull(nl)
                        return (nl, true)
                    nodes[nl].par = u
                nodes[u].sub += c
                return (u, true)
            elif nodes[u].key < x:
                let old_r = nodes[u].right
                let (nr, added) = self.insert_at(old_r, x, c, unique)
                if not added: return (u, false)
                if nr != old_r:
                    nodes[u].right = nr
                    if nodes[nr].pri > nodes[u].pri:
                        let rl = nodes[nr].left
                        nodes[u].right = rl
                        nodes[nr].left = u
                        self.pull(u)
                        self.pull(nr)
                        return (nr, true)
                    nodes[nr].par = u
                nodes[u].sub += c
                return (u, true)
            else:
                if unique: return (u, false)
                nodes[u].cnt += c
                nodes[u].sub += c
                return (u, true)

    fn erase_at(mut self, u: Int, x: K, all: Bool) -> (Int, Int):
        if u < 0: return (-1, 0)
        unchecked:
            if x < nodes[u].key:
                let old_l = nodes[u].left
                let (nl, rem) = self.erase_at(old_l, x, all)
                if rem == 0: return (u, 0)
                if nl != old_l:
                    nodes[u].left = nl
                    if nl >= 0: nodes[nl].par = u
                nodes[u].sub -= rem
                return (u, rem)
            elif nodes[u].key < x:
                let old_r = nodes[u].right
                let (nr, rem) = self.erase_at(old_r, x, all)
                if rem == 0: return (u, 0)
                if nr != old_r:
                    nodes[u].right = nr
                    if nr >= 0: nodes[nr].par = u
                nodes[u].sub -= rem
                return (u, rem)
            else:
                if not all and nodes[u].cnt > 1:
                    nodes[u].cnt -= 1
                    nodes[u].sub -= 1
                    return (u, 1)
                let rem = nodes[u].cnt
                let l = nodes[u].left
                let r = nodes[u].right
                live -= 1
                free.push(u)
                let m = self.merge(l, r)
                return (m, rem)

    fn merge(mut self, a: Int, b: Int) -> Int:
        if a < 0: return b
        if b < 0: return a
        unchecked:
            if nodes[a].pri >= nodes[b].pri:
                nodes[a].right = self.merge(nodes[a].right, b)
                self.pull(a)
                return a
            else:
                nodes[b].left = self.merge(a, nodes[b].left)
                self.pull(b)
                return b

    fn split_lt(mut self, t: Int, k: K) -> (Int, Int):
        if t < 0: return (-1, -1)
        unchecked:
            if nodes[t].key < k:
                let (a, b) = self.split_lt(nodes[t].right, k)
                nodes[t].right = a
                self.pull(t)
                return (t, b)
            let (a, b) = self.split_lt(nodes[t].left, k)
            nodes[t].left = b
            self.pull(t)
            return (a, t)

    fn move_out(mut self, t: Int, mut into: SortedBag<K>):
        if t < 0: return
        unchecked:
            self.move_out(nodes[t].left, mut into)
            into.add_many(nodes[t].key, nodes[t].cnt)
            live -= 1
            free.push(t)
            self.move_out(nodes[t].right, mut into)

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
    /// empty: an empty set
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
    pub fn clear(mut self) = SortedSet(bag.take())

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
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `SortedSet.of([4, 1, 9]).first == Some(1)`
    @get pub fn first(self) -> ?K:
        bag.first

    /// Largest key in the set, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `SortedSet.of([4, 1, 9]).last == Some(9)`
    @get pub fn last(self) -> ?K:
        bag.last

    /// Remove and return the smallest key, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_first() == Some(1) and s.to_list() == [4]`
    pub fn pop_first(mut self) -> ?K = bag.pop_first()

    /// Remove and return the largest key, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_last() == Some(4) and s.to_list() == [1]`
    pub fn pop_last(mut self) -> ?K = bag.pop_last()

    /// Number of keys strictly less than `k`.
    ///
    /// `k` need not be in the set.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `0`
    /// traps: never
    /// example: `SortedSet.of([1, 4, 9]).rank(4) == 1`
    /// example: `SortedSet.of([1, 4, 9]).rank(5) == 2`
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// 0-indexed `i`-th smallest key, or `None` if out of range.
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `SortedSet.of([9, 1, 4]).nth(1) == Some(4)`
    /// example: `SortedSet.of([9, 1, 4]).nth(-1) == None`
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// The sum of the elements in `keys` (all of them by default), for additive `K`:
    /// `set.sum(..b)` adds everything below `b`. \(O(\log n)\): subtree sums are kept.
    ///
    /// `keys` may be any ordered range (`a..b`, `a..=b`, `..b`, `a..`, `..`); its ends need
    /// not be in the set.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `K.zero`, for an empty set or an empty `keys`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `SortedSet.of([1, 2, 5]).sum() == 8`
    /// example: `SortedSet.of([1, 2, 5]).sum(..3) == 3`
    /// example: `SortedSet.of([1, 2, 5]).sum(2..=5) == 7`
    pub fn sum(self, keys: Range<K> = ..) -> K = todo

    /// Smallest key `k` where monotone `ok(k)` holds (`false..false, true..true`), or `None`.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the least key wins
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `SortedSet.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    pub fn first_true(self, ok: K -> Bool) -> ?K:
        bag.first_true(ok)

    /// Largest key `k` where monotone `ok(k)` holds (`true..true, false..false`), or `None`.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the greatest key wins
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
    /// empty: `[]`
    /// order: ascending with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var s = SortedSet.of([1, 2, 5, 8]); s.drain(after: 1, while: x => x < 6) == [2, 5] and s.to_list() == [1, 8]`
    /// example: `var s = SortedSet.of([1, 2, 5, 8]); s.drain(before: 8, while: x => x > 1) == [5, 2]`
    pub fn drain(
        mut self; after: ?K = None,
        before: ?K = None,
        while ok: K -> Bool
    ) -> List<K> = bag.drain(after:, before:, while: ok)

    /// The keys of `self` inside `r`, as an \(O(1)\) view; replaces `first_ge`, `first_gt`, `last_le`,
    /// `last_lt` and `count_range` (decision 103). `r` is any ordered range: `a..b`, `a..=b`, `..b`,
    /// `..=b`, `a..` or `..`. The view lends `self`, so `self` cannot change while it is in use.
    ///
    /// cost: \(O(1)\) to make; see `SortedRange` for each query
    /// empty: a range with no keys gives an empty view
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..7).last == Some(3)`
    pub fn range(self, r: Range<K>) -> SortedRange<K> = todo

    fn range_first(
        self,
        lo: ?K,
        lo_inc: Bool,
        hi: ?K,
        hi_inc: Bool,
    ) -> ?K = bag.range_first(lo, lo_inc, hi, hi_inc)

    fn range_last(
        self,
        lo: ?K,
        lo_inc: Bool,
        hi: ?K,
        hi_inc: Bool,
    ) -> ?K = bag.range_last(lo, lo_inc, hi, hi_inc)

    fn range_len(
        self,
        lo: ?K,
        lo_inc: Bool,
        hi: ?K,
        hi_inc: Bool,
    ) -> Int = bag.range_len(lo, lo_inc, hi, hi_inc)

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        for k in bag.range_each(lo, lo_inc, hi, hi_inc): yield k

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        for k in bag.range_each_rev(lo, lo_inc, hi, hi_inc): yield k

    /// Visit every key in ascending order.
    ///
    /// `for x in s` uses this.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).each().collect() == [1, 2, 3]`
    pub fn each(self) -> yields K:
        for (k, _) in bag.entries(): yield k

    /// Visit every key in descending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: descending
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).each_rev().collect() == [3, 2, 1]`
    pub fn each_rev(self) -> yields K:
        for (k, _) in bag.entries_rev(): yield k

    /// Return all keys in ascending order as a `List<K>`.
    ///
    /// cost: \(O(n)\)
    /// empty: `[]`
    /// order: ascending; a new `List`, keyed from 0
    /// traps: never
    /// example: `SortedSet.of([3, 1, 2]).to_list() == [1, 2, 3]`
    pub fn to_list(self) -> List<K> = bag.to_list()

    /// Preallocate treap node pool for at least `cap` keys.
    ///
    /// cost: \(O(\mathit{cap})\) when the pool grows, \(O(1)\) otherwise
    /// traps: never
    /// example: `var s = SortedSet.of([1]); s.reserve(100); s.to_list() == [1]`
    pub fn reserve(mut self, cap: Int):
        if cap < 0: return

    /// Split off and return all keys `>= k`, keeping `< k` in `self`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's size
    /// empty: both sides empty
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
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a = SortedSet.of([1, 3]); var b = SortedSet.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedSet<K>):
        if bag.distinct < other.bag.distinct:
            let small = bag.take()
            bag = other.bag.take()
            other.bag = small
        for (k, _) in other.bag.entries(): let _ = bag.add_unique(k)
        other.bag.clear()

/// Order-statistic ordered map from `K` to `V` backed by a randomized treap.
/// `get`, `set`, `remove`, `rank`, `nth`, `first_ge`/`last_le`, `first_true`/`last_true`, `split_at`, `join` are \(O(\log n)\).
///
/// Entries are ordered by key; keys are distinct, so no lookup has a tie to break. Every
/// entry-returning method gives `(key, val)`.
pub struct SortedMap<K, V>:
    ks: SortedBag<K>
    vals: HashMap<K, V>

    fn entry(self, k: ?K) -> ?(K, V) = if let x = k: Some((x, vals.get(x)!)) else: None

    fn at(self, c: ?BagCursor) -> ?(K, V) = if let x = c: Some(
        (ks.key_of(x), vals.get(ks.key_of(x))!)
    ) else: None

    /// Create an empty `SortedMap<K, V>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.len == 0`
    pub fn new() -> SortedMap<K, V>:
        SortedMap(SortedBag<K>.new(), HashMap<K, V>())

    /// Preallocate treap node pool for at least `cap` entries.
    ///
    /// cost: \(O(\mathit{cap})\) when the pool grows, \(O(1)\) otherwise
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.reserve(100); m.is_empty`
    pub fn reserve(mut self, cap: Int):
        if cap < 0: return

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
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.clear(); m.get(1) == None`
    pub fn clear(mut self):
        ks.clear()
        vals = HashMap<K, V>()

    /// Move contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); let t = m.take(); m.is_empty and t.get(1) == Some("a")`
    pub fn take(mut self) -> SortedMap<K, V>:
        let t = SortedMap(ks.take(), vals)
        vals = HashMap<K, V>()
        t

    /// Look up `k`, returning `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.get(1) == Some("a") and m.get(2) == None`
    pub fn get(self, k: K) -> ?V:
        vals.get(k)

    /// Whether `k` is present.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.contains(1) and not m.contains(2)`
    pub fn contains(self, k: K) -> Bool:
        vals.get(k) != None

    /// Insert or overwrite `k -> v` (W8 `IndexSet` backing `m[k] = v`).
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m[1] = "a"; m[1] = "b"; m.get(1) == Some("b")`
    pub fn set(mut self, k: K, v: V):
        if vals.get(k) == None: let _ = ks.add(k)
        vals[k] = v

    /// Remove `k` and return its previous value, or `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.remove(1) == Some("a") and m.remove(1) == None`
    pub fn remove(mut self, k: K) -> ?V:
        let v = vals.get(k)
        if v != None:
            let _ = ks.remove(k)
            let _ = vals.remove(k)
        v

    /// Smallest `(key, val)` pair, or `None` if empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.first == Some((1, "a"))`
    @get pub fn first(self) -> ?(K, V):
        self.entry(ks.first)

    /// Largest `(key, val)` pair, or `None` if empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.last == Some((3, "c"))`
    @get pub fn last(self) -> ?(K, V):
        self.entry(ks.last)

    /// Remove and return the smallest `(key, val)` pair, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.pop_first() == Some((1, "a")) and m.len == 1`
    pub fn pop_first(mut self) -> ?(K, V):
        let e = self.entry(ks.first)
        if let (k, _) = e: let _ = self.remove(k)
        e

    /// Remove and return the largest `(key, val)` pair, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.pop_last() == Some((3, "c")) and m.len == 1`
    pub fn pop_last(mut self) -> ?(K, V):
        let e = self.entry(ks.last)
        if let (k, _) = e: let _ = self.remove(k)
        e

    /// Smallest entry with key `>= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.first_ge(2) == Some((3, "c")) and m.first_ge(3) == Some((3, "c"))`
    pub fn first_ge(self, k: K) -> ?(K, V):
        self.at(ks.cursor_ge(k))

    /// Smallest entry with key `> k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.first_gt(1) == Some((3, "c")) and m.first_gt(3) == None`
    pub fn first_gt(self, k: K) -> ?(K, V):
        self.at(ks.cursor_gt(k))

    /// Largest entry with key `<= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.last_le(2) == Some((1, "a")) and m.last_le(0) == None`
    pub fn last_le(self, k: K) -> ?(K, V):
        self.at(ks.cursor_le(k))

    /// Largest entry with key `< k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.last_lt(3) == Some((1, "a")) and m.last_lt(1) == None`
    pub fn last_lt(self, k: K) -> ?(K, V):
        self.at(ks.cursor_lt(k))

    /// Number of entries with key strictly less than `k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `0`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.rank(3) == 1 and m.rank(9) == 2`
    pub fn rank(self, k: K) -> Int:
        ks.rank(k)

    /// The `i`-th entry in key order (0-based), or `None`.
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.nth(1) == Some((3, "c")) and m.nth(2) == None`
    pub fn nth(self, i: Int) -> ?(K, V):
        self.entry(ks.nth(i))

    /// Smallest entry where monotone `ok(k, v)` becomes true, or `None`.
    ///
    /// `ok` must be false-then-true along ascending keys.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the least key wins
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `var m: SortedMap<Int, Int> = SortedMap(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.first_true((k, v) => v >= 30) == Some((4, 40))`
    pub fn first_true(self, ok: (K, V) -> Bool) -> ?(K, V):
        self.entry(ks.first_true(k => ok(k, vals.get(k)!)))

    /// Largest entry where monotone `ok(k, v)` still holds, or `None`.
    ///
    /// `ok` must be true-then-false along ascending keys.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `None`
    /// ties: none: keys are distinct; the greatest key wins
    /// traps: never; an `ok` that is not true-then-false gives an unspecified answer
    /// example: `var m: SortedMap<Int, Int> = SortedMap(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.last_true((k, v) => k <= 5) == Some((4, 40))`
    pub fn last_true(self, ok: (K, V) -> Bool) -> ?(K, V):
        self.entry(ks.last_true(k => ok(k, vals.get(k)!)))

    /// Remove and return entries after `after` (ascending) or before `before` (descending) while `ok` holds.
    ///
    /// With neither bound the walk starts at the smallest key. It stops at the first entry
    /// where `ok` is false, which stays; bounds are exclusive. Giving both is an error.
    ///
    /// cost: \(O(k \log n)\) expected for `k` removed entries
    /// empty: `[]`
    /// order: ascending key with `after` or no bound, descending with `before`; a new `List`, keyed from 0
    /// traps: when both `after` and `before` are given
    /// example: `var m: SortedMap<Int, Int> = SortedMap(); m.set(1, 10); m.set(4, 40); m.set(6, 60); m.drain(after: 1, while: (k, v) => v < 50) == [(4, 40)] and m.len == 2`
    pub fn drain(
        mut self; after: ?K = None,
        before: ?K = None,
        while ok: (K, V) -> Bool
    ) -> List<(K, V)>:
        let gone = ks.drain(after:, before:, while: k => ok(k, vals.get(k)!))
        let out: List<(K, V)> = [(k, vals.remove(k)!) for k in gone]
        out

    /// Split off and return all entries with key `>= k`.
    ///
    /// `self` keeps the entries with key `< k`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected for the keys, plus \(O(m)\) expected to move the `m` values split off
    /// empty: both sides empty
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); let hi = m.split_at(2); m.len == 1 and hi.first == Some((3, "c"))`
    pub fn split_at(mut self, k: K) -> SortedMap<K, V>:
        let hks = ks.split_at(k)
        var hv = HashMap<K, V>()
        for (x, _) in hks.entries(): hv[x] = vals.remove(x)!
        SortedMap(hks, hv)

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The precondition is checked in \(O(1)\) against the last key of `self` and the first of
    /// `other`.
    ///
    /// cost: as `SortedBag.join` for the keys, plus \(O(m)\) expected to move the `m` values of `other`
    /// traps: when both are non-empty and the last key of `self` is `>=` the first key of `other`
    /// example: `var a: SortedMap<Int, Str> = SortedMap(); a.set(1, "a"); var b: SortedMap<Int, Str> = SortedMap(); b.set(3, "c"); a.join(b); a.len == 2`
    pub fn join(mut self, other: SortedMap<K, V>):
        ks.join(other.ks)
        for (x, _) in other.ks.entries(): vals[x] = other.vals.get(x)!

    /// Merge all entries of `other` into `self` (emptying `other`), combining collisions with `join` or overwriting.
    ///
    /// For a key in both maps the result is `join(self's value, other's value)`, or `other`'s
    /// value when `join` is `None`, whichever map is drained into the other.
    ///
    /// cost: \(O(\min(n, m) \log(n + m))\) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a: SortedMap<Int, Int> = SortedMap(); a.set(1, 10); var b: SortedMap<Int, Int> = SortedMap(); b.set(1, 5); b.set(2, 7); a.absorb(mut b, join: (x, y) => x + y); a.get(1) == Some(15) and a.get(2) == Some(7) and b.is_empty`
    pub fn absorb(mut self, mut other: SortedMap<K, V>, join: ?((V, V) -> V) = None):
        var small = other.take()
        var flip = false
        if small.len > self.len:
            let big = small
            small = self.take()
            ks = big.ks
            vals = big.vals
            flip = true
        for (k, v) in small.items:
            if let old = vals.get(k):
                if let j = join: vals[k] = if flip: j(v, old) else: j(old, v)
                elif not flip: vals[k] = v
            else:
                let _ = ks.add(k)
                vals[k] = v

    /// \(O(1)\) to make. Yields `(key, val)` pairs in ascending key order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.items.collect() == [(1, "a"), (3, "c")]`
    @get pub fn items(self) -> yields (K, V):
        for k in ks.entries(): yield (k.key, vals.get(k.key)!)

    /// Yield `(key, val)` pairs in descending key order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: descending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.each_rev().collect() == [(3, "c"), (1, "a")]`
    pub fn each_rev(self) -> yields (K, V):
        for k in ks.entries_rev(): yield (k.key, vals.get(k.key)!)

    ///
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.keys.collect() == [1, 3]`
    @get pub fn keys(self) -> yields K:
        for k in ks.entries(): yield k.key

    ///
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.values.collect() == ["a", "c"]`
    @get pub fn values(self) -> yields V:
        for k in ks.entries(): yield vals.get(k.key)!

/// Monoid-augmented ordered multiset maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Supports \(O(\log n)\) `add`, `remove`, `rank`, `nth`, `fold_lt`, `fold_le`, `fold_ge`, `fold_gt`, `fold_between`, and \(O(1)\) `all`.
///
/// Every fold combines `leaf(k)` over the chosen elements in ascending key order, one `leaf(k)`
/// per copy, starting from `id`; `op` need not be commutative. An empty selection folds to `id`.
pub struct AugSortedBag<K, T>:
    _len: Int
    op: (T, T) -> T
    id: T
    leaf: K -> T
    pub keys: List<K> = []
    pri: List<Int> = []
    ls: List<Int> = []
    rs: List<Int> = []
    sz: List<Int> = []
    agg: List<T> = []
    root: Int = -1
    seed: Int = 1
    lo_k: ?K = None
    hi_k: ?K = None

    fn sub(self, t: Int) -> T = if t < 0: id else: agg[t]
    fn size(self, t: Int) -> Int = if t < 0: 0 else: sz[t]

    fn pull(mut self, t: Int):
        sz[t] = self.size(ls[t]) + 1 + self.size(rs[t])
        agg[t] = op(op(self.sub(ls[t]), leaf(keys[t])), self.sub(rs[t]))

    // (keys < x, keys >= x)
    fn split(mut self, t: Int, x: K) -> (Int, Int):
        if t < 0: return (-1, -1)
        if keys[t] < x:
            let (a, b) = self.split(rs[t], x)
            rs[t] = a
            self.pull(t)
            (t, b)
        else:
            let (a, b) = self.split(ls[t], x)
            ls[t] = b
            self.pull(t)
            (a, t)

    fn merge(mut self, a: Int, b: Int) -> Int:
        if a < 0: return b
        if b < 0: return a
        if pri[a] > pri[b]:
            rs[a] = self.merge(rs[a], b)
            self.pull(a)
            a
        else:
            ls[b] = self.merge(a, ls[b])
            self.pull(b)
            b

    fn drop_min(mut self, t: Int) -> Int:
        if ls[t] < 0: return rs[t]
        ls[t] = self.drop_min(ls[t])
        self.pull(t)
        t

    fn extreme(self, right: Bool) -> ?K:
        var t = root
        if t < 0: return None
        while (if right: rs[t] else: ls[t]) >= 0: t = if right: rs[t] else: ls[t]
        Some(keys[t])

    fn rank_le(self, k: K) -> Int:
        var (t, n) = (root, 0)
        while t >= 0:
            if keys[t] <= k:
                n += self.size(ls[t]) + 1
                t = rs[t]
            else: t = ls[t]
        n

    fn fold_from(self, start: Int, lo: K, strict: Bool) -> T:
        var (t, res) = (start, id)
        while t >= 0:
            if keys[t] > lo or (not strict and keys[t] == lo):
                res = op(op(leaf(keys[t]), self.sub(rs[t])), res)
                t = ls[t]
            else: t = rs[t]
        res

    fn fold_upto(self, start: Int, hi: K, strict: Bool) -> T:
        var (t, res) = (start, id)
        while t >= 0:
            if keys[t] < hi or (not strict and keys[t] == hi):
                res = op(res, op(self.sub(ls[t]), leaf(keys[t])))
                t = rs[t]
            else: t = ls[t]
        res

    /// Create an empty `AugSortedBag<K, T>` with associative `op`, identity `id`, and key map `leaf`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.all == 0`
    pub fn new(
        op: (T, T) -> T,
        id: T,
        leaf: K -> T,
    ) -> AugSortedBag<K, T> = AugSortedBag<K, T>(_len: 0, op:, id:, leaf:)

    /// Preallocate treap node pool for at least `cap` elements.
    ///
    /// cost: \(O(\mathit{cap})\) when the pool grows, \(O(1)\) otherwise
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.reserve(100); b.len == 0`
    pub fn reserve(mut self, cap: Int):
        assert cap >= 0

    /// Total number of elements (with multiplicity).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.len == 2`
    @get pub fn len(self) -> Int = _len

    /// Whether the bag is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.is_empty`
    @get pub fn is_empty(self) -> Bool = _len == 0

    /// Fold of all elements in the bag.
    ///
    /// cost: \(O(1)\)
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.all == 7`
    @get pub fn all(self) -> T = self.sub(root)

    /// Insert one copy of `x`.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.count(3) == 2`
    pub fn add(mut self, x: K):
        seed = (seed * 1103515245 + 12345) % 2147483648
        keys.push(x)
        pri.push(seed)
        ls.push(-1)
        rs.push(-1)
        sz.push(1)
        agg.push(leaf(x))
        let (a, b) = self.split(root, x)
        let m = self.merge(a, keys.len - 1)
        root = self.merge(m, b)
        _len += 1
        if lo_k == None or x < lo_k!: lo_k = Some(x)
        if hi_k == None or hi_k! < x: hi_k = Some(x)

    /// Remove one copy of `x`; returns false if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.remove(3) and b.all == 4 and not b.remove(2)`
    pub fn remove(mut self, x: K) -> Bool:
        if self.count(x) == 0: return false
        let (a, b) = self.split(root, x)
        let c = self.drop_min(b)
        root = self.merge(a, c)
        _len -= 1
        lo_k = self.extreme(false)
        hi_k = self.extreme(true)
        true

    /// Multiplicity of `x`.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(3); b.count(3) == 2 and b.count(1) == 0`
    pub fn count(self, x: K) -> Int = self.rank_le(x) - self.rank(x)

    /// Number of elements strictly less than `k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.rank(3) == 1`
    pub fn rank(self, k: K) -> Int:
        var (t, n) = (root, 0)
        while t >= 0:
            if keys[t] < k:
                n += self.size(ls[t]) + 1
                t = rs[t]
            else: t = ls[t]
        n

    /// The `i`-th smallest element (0-based), or `None`.
    ///
    /// Copies count; `None` for any `i` outside `0..len`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.nth(2) == Some(3) and b.nth(3) == None`
    pub fn nth(self, i: Int) -> ?K:
        if i < 0 or i >= _len: return None
        var (t, j) = (root, i)
        while true:
            let l = self.size(ls[t])
            if j < l: t = ls[t]
            elif j == l: return Some(keys[t])
            else:
                j -= l + 1
                t = rs[t]
        None

    /// Smallest element, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: copies of the least key are equal
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.first == Some(1)`
    @get pub fn first(self) -> ?K = lo_k

    /// Largest element, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: copies of the greatest key are equal
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.last == Some(3)`
    @get pub fn last(self) -> ?K = hi_k

    /// Fold elements with key `< hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_lt(3) == 1`
    pub fn fold_lt(self, hi: K) -> T = self.fold_upto(root, hi, true)

    /// Fold elements with key `<= hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_le(3) == 7`
    pub fn fold_le(self, hi: K) -> T = self.fold_upto(root, hi, false)

    /// Fold elements with key `>= lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_ge(2) == 6`
    pub fn fold_ge(self, lo: K) -> T = self.fold_from(root, lo, false)

    /// Fold elements with key `> lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_gt(1) == 6 and b.fold_gt(3) == 0`
    pub fn fold_gt(self, lo: K) -> T = self.fold_from(root, lo, true)

    /// Fold elements with key in `[lo, hi)`.
    ///
    /// `lo >= hi` selects nothing and gives `id`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`, also when `lo >= hi`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_between(1, 3) == 1 and b.fold_between(2, 4) == 6`
    pub fn fold_between(self, lo: K, hi: K) -> T:
        if not (lo < hi): return id
        var t = root
        while t >= 0 and not (lo <= keys[t] and keys[t] < hi):
            t = if keys[t] < lo: rs[t] else: ls[t]
        if t < 0: return id
        op(op(self.fold_from(ls[t], lo, false), leaf(keys[t])), self.fold_upto(rs[t], hi, true))

    /// Yield all elements in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending, each key repeated by its count
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.each().collect() == [1, 3, 3]`
    pub fn each(self) -> yields K:
        var stack: List<Int> = []
        var t = root
        while t >= 0 or stack.len > 0:
            while t >= 0:
                stack.push(t)
                t = ls[t]
            let u = stack.pop()!
            yield keys[u]
            t = rs[u]

/// Monoid-augmented ordered unique set maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Covers dynamic range sums and consecutive-gap monoids (`cf2066e`) in \(O(\log n)\).
///
/// Every fold combines `leaf(k)` over the chosen keys in ascending order, starting from `id`;
/// `op` need not be commutative. An empty selection folds to `id`.
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

    /// Preallocate treap node pool for at least `cap` keys.
    ///
    /// cost: \(O(\mathit{cap})\) when the pool grows, \(O(1)\) otherwise
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
    /// empty: `id`
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
    /// empty: `0`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.rank(3) == 1 and s.rank(4) == 2`
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// The `i`-th smallest key (0-based), or `None`.
    ///
    /// `None` for any `i` outside `0..len`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.nth(1) == Some(3) and s.nth(2) == None`
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// Smallest key, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.first == Some(1)`
    @get pub fn first(self) -> ?K = bag.first

    /// Largest key, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: none: keys are distinct
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.last == Some(3)`
    @get pub fn last(self) -> ?K = bag.last

    /// Fold keys `< hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_lt(5) == 4`
    pub fn fold_lt(self, hi: K) -> T = bag.fold_lt(hi)

    /// Fold keys `<= hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_le(5) == 9`
    pub fn fold_le(self, hi: K) -> T = bag.fold_le(hi)

    /// Fold keys `>= lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_ge(3) == 8`
    pub fn fold_ge(self, lo: K) -> T = bag.fold_ge(lo)

    /// Fold keys `> lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_gt(3) == 5`
    pub fn fold_gt(self, lo: K) -> T = bag.fold_gt(lo)

    /// Fold keys in `[lo, hi)`.
    ///
    /// `lo >= hi` selects nothing and gives `id`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `id`, also when `lo >= hi`
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(1); s.add(3); s.add(5); s.fold_between(1, 5) == 4 and s.fold_between(4, 2) == 0`
    pub fn fold_between(self, lo: K, hi: K) -> T = bag.fold_between(lo, hi)

    /// Yield all keys in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: yields nothing
    /// order: ascending
    /// traps: never
    /// example: `var s: AugSortedSet<Int, Int> = AugSortedSet((x, y) => x + y, 0, k => k); s.add(3); s.add(1); s.each().collect() == [1, 3]`
    pub fn each(self) -> yields K:
        for x in bag.each(): yield x

/// \(O(1)\) point add, \(O(\sqrt{N})\) k-th order statistic over an integer value range `r`.
/// Designed as a Mo's algorithm companion for value-range frequency tables.
///
/// `N` is `r.len`; the table keeps one count per value and one per block of about `sqrt(N)`
/// values. Writes outside `r` trap; reads outside `r` see a count of 0.
pub struct BucketCount:
    lo: Int
    hi: Int
    bs: Int
    total: Int
    cnt: List<Int>
    blk: List<Int>

    /// \(O(r.\mathit{len})\). Create an empty bucketed frequency table over `r`.
    ///
    /// cost: \(O(r.\mathit{len})\)
    /// empty: an empty `r` gives a table every write traps on
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.len == 0`
    pub fn new(r: Range<Int>) -> BucketCount:
        let n = max(r.len, 0)
        var bs = 1
        while bs * bs < n: bs += 1
        BucketCount(r.start, r.start + n, bs, 0, List.filled(n, 0), List.filled(n / bs + 1, 0))

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
        blk[i / bs] += delta
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
    /// empty: `0`
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(3); b.add(7, 2); b.count_lt(7) == 1 and b.count_lt(8) == 3`
    pub fn count_lt(self, x: Int) -> Int:
        if x <= lo: return 0
        if x >= hi: return total
        let i = x - lo
        var acc = 0
        for b in 0..i / bs: acc += blk[b]
        for j in i / bs * bs..i: acc += cnt[j]
        acc

    /// \(O(\sqrt{N})\). The `k`-th smallest element, 0-based like `nth`: `kth(0)` is the smallest.
    ///
    /// Copies count: with elements `3, 7, 7`, `kth(1)` and `kth(2)` are both `7`.
    ///
    /// cost: \(O(\sqrt{N})\)
    /// ties: none: the `k`-th element is one value
    /// traps: when `k` is not in `0..len`
    /// example: `var b = BucketCount(0..10); b.add(3); b.add(7, 2); b.kth(0) == 3 and b.kth(1) == 7 and b.kth(2) == 7`
    pub fn kth(self, k: Int) -> Int:
        assert 0 <= k < total, "`k` is outside `0..len`"
        var left = k
        var b = 0
        while left >= blk[b]:
            left -= blk[b]
            b += 1
        var j = b * bs
        while left >= cnt[j]:
            left -= cnt[j]
            j += 1
        lo + j

    ///
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

    assert b.range(2..).first == 2 and b.range(3..).first == 5 and b.range(10..).first == None and b.range(10..).first == None
    assert b.range(..=5).last == 5 and b.range(..5).last == 2 and b.range(..=0).last == None and b.range(..1).last == None
    assert b.rank(1) == 0 and b.rank(2) == 1 and b.rank(3) == 3 and b.rank(5) == 3 and b.rank(6) == 6 and b.rank(10) == 7
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
            var lt = 0
            for v in plain:
                if v < x: lt += 1
            assert b.rank(x) == lt
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
    var brute = 0
    for i in 0..a.len:
        for j in i + 1..a.len:
            if a[i] > a[j]: brute += 1
    assert inv == brute
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
    assert b.key_of(b.cursor_le(3)!) == 1 and b.key_of(b.cursor_lt(4)!) == 1 and b.cursor_lt(1) == None
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

/// The keys of a `SortedBag` or `SortedSet` inside a range, made by `range(r)`. A view: it stores
/// the bounds and lends the container, and allocates nothing. `for x in s.range(r)` iterates
/// `each()`. Copies of one key in a `SortedBag` are all inside or all outside, so nothing here
/// has a tie to break. `n` is the container's `len`, `k` the number of keys visited.
pub struct SortedRange<K>:
    lo: ?K
    lo_inc: Bool
    hi: ?K
    hi_inc: Bool

    /// The smallest key in the range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// ties: copies of a key are equal, so any is the answer
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(2..).first == Some(3)`
    @get pub fn first(self) -> ?K = todo

    /// The largest key in the range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `None`
    /// ties: copies of a key are equal, so any is the answer
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..=6).last == Some(3)`
    @get pub fn last(self) -> ?K = todo

    /// The number of keys in the range, counting copies.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: 0
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(3..=3).len == 2`
    @get pub fn len(self) -> Int = todo

    /// True when the range holds no key.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `true`
    /// traps: never
    /// example: `SortedSet.of([1, 7]).range(2..7).is_empty`
    @get pub fn is_empty(self) -> Bool = todo

    /// The keys in ascending order, copies included.
    ///
    /// cost: \(O(\log n + k)\) expected for `k` keys visited
    /// empty: yields nothing
    /// order: ascending
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(2..).each().collect() == [3, 3, 7]`
    pub fn each(self) -> yields K = todo

    /// The keys in descending order, copies included.
    ///
    /// cost: \(O(\log n + k)\) expected for `k` keys visited
    /// empty: yields nothing
    /// order: descending
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..7).rev.collect() == [3, 1]`
    @get pub fn rev(self) -> yields K = todo

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
