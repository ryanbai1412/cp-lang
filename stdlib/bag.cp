//! Ordered multiset and set with order statistics and cursors: `SortedBag<K>`, `OrderedSet<K>`.

use std.seq.{remove}

/// Read-only handle into a `SortedBag<K>` node, valid until the next mutation.
pub struct BagCursor(id: Int)

/// An ordered multiset (bag) of `K` backed by a randomized treap over flat arrays
/// with recycled node slots and parent pointers for O(1) amortized cursor stepping.
/// Keeps one node per distinct key with its multiplicity and subtree element count.
/// `add`, `add_many`, `remove`, `remove_all`, `contains`, `count`, `first`, `last`,
/// `pop_first`, `pop_last`, `first_ge`, `first_gt`, `last_le`, `last_lt`, `rank`,
/// `count_range`, `nth`, `first_where`, and `cursor_*` are O(log n) expected;
/// `key_of` and `count_of` are O(1); `next_cursor` and `prev_cursor` are O(1) amortized;
/// `len`, `distinct`, `is_empty`, `clear`, and `take` are O(1);
/// `entries`, `entries_rev`, and `to_list` are O(n).
pub struct SortedBag<K>:
    key: List<K>
    pri: List<Int>
    cnt: List<Int>
    sub: List<Int>
    left: List<Int>
    right: List<Int>
    par: List<Int>
    free: List<Int>
    live: Int
    root: Int

    /// Create an empty `SortedBag<K>`. O(1).
    pub fn new() -> SortedBag<K> = SortedBag([], [], [], [], [], [], [], [], 0, -1)

    /// Build a `SortedBag` containing every element of `xs`. O(n log n) expected.
    pub fn of(xs: List<K>) -> SortedBag<K>:
        var b = SortedBag<K>.new()
        for x in xs: b.add(x)
        b

    /// Total number of elements in the bag (counting duplicates). O(1).
    pub @get fn len(self) -> Int = self.sub_of(root)

    /// Number of distinct keys currently in the bag. O(1).
    pub @get fn distinct(self) -> Int = live

    /// Whether the bag has no elements. O(1).
    pub @get fn is_empty(self) -> Bool = self.len == 0

    /// Remove all elements from the bag. O(1).
    pub fn clear(mut self):
        key = []
        pri = []
        cnt = []
        sub = []
        left = []
        right = []
        par = []
        free = []
        live = 0
        root = -1

    /// Move the bag's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> SortedBag<K>:
        let out = SortedBag(key, pri, cnt, sub, left, right, par, free, live, root)
        key = []
        pri = []
        cnt = []
        sub = []
        left = []
        right = []
        par = []
        free = []
        live = 0
        root = -1
        out

    /// Insert one copy of `x` into the bag. Always returns `true`. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool:
        let (lo, rest) = self.split_lt(root, x)
        let (eq, hi) = self.split_le(rest, x)
        var mid = eq
        if mid >= 0:
            cnt[mid] += 1
            self.pull(mid)
        else:
            mid = self.make_node(x, 1)
        let lm = self.merge(lo, mid)
        root = self.merge(lm, hi)
        if root >= 0: par[root] = -1
        true

    /// Insert `c >= 0` copies of `x` into the bag. O(log n) expected.
    pub fn add_many(mut self, x: K, c: Int):
        assert c >= 0
        if c == 0: return
        let (lo, rest) = self.split_lt(root, x)
        let (eq, hi) = self.split_le(rest, x)
        var mid = eq
        if mid >= 0:
            cnt[mid] += c
            self.pull(mid)
        else:
            mid = self.make_node(x, c)
        let lm = self.merge(lo, mid)
        root = self.merge(lm, hi)
        if root >= 0: par[root] = -1

    /// Remove one copy of `x` if present; returns `true` if a copy was removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool:
        let (lo, rest) = self.split_lt(root, x)
        let (eq, hi) = self.split_le(rest, x)
        if eq < 0:
            root = self.merge(lo, hi)
            if root >= 0: par[root] = -1
            return false
        var mid = -1
        if cnt[eq] > 1:
            cnt[eq] -= 1
            self.pull(eq)
            mid = eq
        else:
            live -= 1
            free.push(eq)
        let lm = self.merge(lo, mid)
        root = self.merge(lm, hi)
        if root >= 0: par[root] = -1
        true

    /// Remove all copies of `x` and return how many were removed. O(log n) expected.
    pub fn remove_all(mut self, x: K) -> Int:
        let (lo, rest) = self.split_lt(root, x)
        let (eq, hi) = self.split_le(rest, x)
        if eq < 0:
            root = self.merge(lo, hi)
            if root >= 0: par[root] = -1
            return 0
        let removed = cnt[eq]
        live -= 1
        free.push(eq)
        root = self.merge(lo, hi)
        if root >= 0: par[root] = -1
        removed

    /// How many copies of `x` are currently in the bag. O(log n) expected.
    pub fn count(self, x: K) -> Int:
        var u = root
        while u >= 0:
            if x < key[u]: u = left[u]
            elif key[u] < x: u = right[u]
            else: return cnt[u]
        0

    /// Whether at least one copy of `x` is in the bag. O(log n) expected.
    pub fn contains(self, x: K) -> Bool = self.count(x) > 0

    /// The smallest element, or `none` when empty. O(log n) expected.
    pub @get fn first(self) -> ?K:
        if root < 0: return none
        var u = root
        while left[u] >= 0: u = left[u]
        key[u]

    /// The largest element, or `none` when empty. O(log n) expected.
    pub @get fn last(self) -> ?K:
        if root < 0: return none
        var u = root
        while right[u] >= 0: u = right[u]
        key[u]

    /// Remove and return one copy of the smallest element, or `none` when empty. O(log n) expected.
    pub fn pop_first(mut self) -> ?K:
        let x = self.first
        if x == none: return none
        self.remove(x!)
        x

    /// Remove and return one copy of the largest element, or `none` when empty. O(log n) expected.
    pub fn pop_last(mut self) -> ?K:
        let x = self.last
        if x == none: return none
        self.remove(x!)
        x

    /// Smallest element `>= k`, or `none`. O(log n) expected.
    pub fn first_ge(self, k: K) -> ?K:
        var ans: ?K = none
        var u = root
        while u >= 0:
            if not (key[u] < k):
                ans = key[u]
                u = left[u]
            else:
                u = right[u]
        ans

    /// Smallest element `> k`, or `none`. O(log n) expected.
    pub fn first_gt(self, k: K) -> ?K:
        var ans: ?K = none
        var u = root
        while u >= 0:
            if k < key[u]:
                ans = key[u]
                u = left[u]
            else:
                u = right[u]
        ans

    /// Largest element `<= k`, or `none`. O(log n) expected.
    pub fn last_le(self, k: K) -> ?K:
        var ans: ?K = none
        var u = root
        while u >= 0:
            if not (k < key[u]):
                ans = key[u]
                u = right[u]
            else:
                u = left[u]
        ans

    /// Largest element `< k`, or `none`. O(log n) expected.
    pub fn last_lt(self, k: K) -> ?K:
        var ans: ?K = none
        var u = root
        while u >= 0:
            if key[u] < k:
                ans = key[u]
                u = right[u]
            else:
                u = left[u]
        ans

    /// Number of elements strictly less than `k` (counting duplicates). O(log n) expected.
    pub fn rank(self, k: K) -> Int:
        var ans = 0
        var u = root
        while u >= 0:
            if key[u] < k:
                ans += self.sub_of(left[u]) + cnt[u]
                u = right[u]
            else:
                u = left[u]
        ans

    /// Number of elements in `[lo, hi)` (counting duplicates; 0 when `hi <= lo`). O(log n) expected.
    pub fn count_range(self, lo: K, hi: K) -> Int:
        if not (lo < hi): return 0
        self.rank(hi) - self.rank(lo)

    /// 0-indexed `i`-th smallest element (counting duplicates), or `none` if out of bounds. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K:
        if i < 0 or i >= self.len: return none
        var k = i
        var u = root
        while u >= 0:
            let lsz = self.sub_of(left[u])
            if k < lsz:
                u = left[u]
            elif k < lsz + cnt[u]:
                return key[u]
            else:
                k -= lsz + cnt[u]
                u = right[u]
        none

    /// Smallest key where monotone `ok(k)` is true (`false..false, true..true`), or `none`. O(log n) expected.
    pub fn first_where(self, ok: K -> Bool) -> ?K:
        var ans: ?K = none
        var u = root
        while u >= 0:
            if ok(key[u]):
                ans = key[u]
                u = left[u]
            else:
                u = right[u]
        ans

    /// Cursor to the smallest key, or `none` when empty. O(log n) expected.
    pub fn cursor_first(self) -> ?BagCursor:
        if root < 0: return none
        var u = root
        while left[u] >= 0: u = left[u]
        BagCursor(u)

    /// Cursor to the largest key, or `none` when empty. O(log n) expected.
    pub fn cursor_last(self) -> ?BagCursor:
        if root < 0: return none
        var u = root
        while right[u] >= 0: u = right[u]
        BagCursor(u)

    /// Cursor to exact key `k`, or `none` if absent. O(log n) expected.
    pub fn cursor_at(self, k: K) -> ?BagCursor:
        var u = root
        while u >= 0:
            if k < key[u]: u = left[u]
            elif key[u] < k: u = right[u]
            else: return BagCursor(u)
        none

    /// Cursor to smallest key `>= k`, or `none`. O(log n) expected.
    pub fn cursor_ge(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        while u >= 0:
            if not (key[u] < k):
                ans = u
                u = left[u]
            else:
                u = right[u]
        if ans >= 0: BagCursor(ans) else: none

    /// Cursor to smallest key `> k`, or `none`. O(log n) expected.
    pub fn cursor_gt(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        while u >= 0:
            if k < key[u]:
                ans = u
                u = left[u]
            else:
                u = right[u]
        if ans >= 0: BagCursor(ans) else: none

    /// Cursor to largest key `<= k`, or `none`. O(log n) expected.
    pub fn cursor_le(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        while u >= 0:
            if not (k < key[u]):
                ans = u
                u = right[u]
            else:
                u = left[u]
        if ans >= 0: BagCursor(ans) else: none

    /// Cursor to largest key `< k`, or `none`. O(log n) expected.
    pub fn cursor_lt(self, k: K) -> ?BagCursor:
        var ans = -1
        var u = root
        while u >= 0:
            if key[u] < k:
                ans = u
                u = right[u]
            else:
                u = left[u]
        if ans >= 0: BagCursor(ans) else: none

    /// Key stored at cursor `c`. O(1).
    pub fn key_of(self, c: BagCursor) -> K = key[c.id]

    /// Multiplicity stored at cursor `c`. O(1).
    pub fn count_of(self, c: BagCursor) -> Int = cnt[c.id]

    /// Cursor to the next larger distinct key, or `none`. O(1) amortized.
    pub fn next_cursor(self, c: BagCursor) -> ?BagCursor:
        var u = c.id
        if right[u] >= 0:
            u = right[u]
            while left[u] >= 0: u = left[u]
            return BagCursor(u)
        while par[u] >= 0:
            let p = par[u]
            if left[p] == u: return BagCursor(p)
            u = p
        none

    /// Cursor to the next smaller distinct key, or `none`. O(1) amortized.
    pub fn prev_cursor(self, c: BagCursor) -> ?BagCursor:
        var u = c.id
        if left[u] >= 0:
            u = left[u]
            while right[u] >= 0: u = right[u]
            return BagCursor(u)
        while par[u] >= 0:
            let p = par[u]
            if right[p] == u: return BagCursor(p)
            u = p
        none

    /// Visit every distinct `(key, count)` in ascending order. O(1) amortized per step.
    pub fn entries(self) -> yields (key: K, count: Int):
        var cur = self.cursor_first()
        while let c = cur:
            yield (key: key[c.id], count: cnt[c.id])
            cur = self.next_cursor(c)

    /// Visit every distinct `(key, count)` in descending order. O(1) amortized per step.
    pub fn entries_rev(self) -> yields (key: K, count: Int):
        var cur = self.cursor_last()
        while let c = cur:
            yield (key: key[c.id], count: cnt[c.id])
            cur = self.prev_cursor(c)

    /// Every element in ascending order, repeated by multiplicity. O(n).
    pub fn to_list(self) -> List<K>:
        var out: List<K> = []
        for (k, c) in self.entries():
            for _ in 0..c: out.push(k)
        out

    fn sub_of(self, t: Int) -> Int = if t < 0: 0 else: sub[t]

    fn pull(mut self, u: Int):
        sub[u] = cnt[u] + self.sub_of(left[u]) + self.sub_of(right[u])
        if left[u] >= 0: par[left[u]] = u
        if right[u] >= 0: par[right[u]] = u

    fn make_node(mut self, x: K, c: Int) -> Int:
        live += 1
        let p = rand(0, 1 << 60)
        if let u = free.pop():
            key[u] = x
            pri[u] = p
            cnt[u] = c
            sub[u] = c
            left[u] = -1
            right[u] = -1
            par[u] = -1
            return u
        key.push(x)
        pri.push(p)
        cnt.push(c)
        sub.push(c)
        left.push(-1)
        right.push(-1)
        par.push(-1)
        key.len - 1

    /// Split `t` into `(keys < x, keys >= x)`.
    fn split_lt(mut self, t: Int, x: K) -> (Int, Int):
        if t < 0: return (-1, -1)
        if key[t] < x:
            let (a, b) = self.split_lt(right[t], x)
            right[t] = a
            self.pull(t)
            (t, b)
        else:
            let (a, b) = self.split_lt(left[t], x)
            left[t] = b
            self.pull(t)
            (a, t)

    /// Split `t` into `(keys <= x, keys > x)`.
    fn split_le(mut self, t: Int, x: K) -> (Int, Int):
        if t < 0: return (-1, -1)
        if not (x < key[t]):
            let (a, b) = self.split_le(right[t], x)
            right[t] = a
            self.pull(t)
            (t, b)
        else:
            let (a, b) = self.split_le(left[t], x)
            left[t] = b
            self.pull(t)
            (a, t)

    fn merge(mut self, a: Int, b: Int) -> Int:
        if a < 0: return b
        if b < 0: return a
        if pri[a] >= pri[b]:
            right[a] = self.merge(right[a], b)
            self.pull(a)
            a
        else:
            left[b] = self.merge(a, left[b])
            self.pull(b)
            b

/// An ordered set of unique keys `K` with order statistics (`rank`, `count_range`, `nth`)
/// and monotone search (`first_where`). Backed by `SortedBag<K>`.
/// All single-key operations are O(log n) expected; `len`, `is_empty`, `clear`, `take` are O(1);
/// `each`, `each_rev`, and `to_list` are O(n).
pub struct OrderedSet<K>:
    bag: SortedBag<K>

    /// Create an empty `OrderedSet<K>`. O(1).
    pub fn new() -> OrderedSet<K> = OrderedSet(SortedBag<K>.new())

    /// Build an `OrderedSet` containing the distinct elements of `xs`. O(n log n) expected.
    pub fn of(xs: List<K>) -> OrderedSet<K>:
        var s = OrderedSet<K>.new()
        for x in xs: let _ = s.add(x)
        s

    /// Number of keys in the set. O(1).
    pub @get fn len(self) -> Int = bag.len

    /// Whether the set is empty. O(1).
    pub @get fn is_empty(self) -> Bool = bag.is_empty

    /// Remove all keys from the set. O(1).
    pub fn clear(mut self): bag.clear()

    /// Move all keys out, leaving `self` empty. O(1).
    pub fn take(mut self) -> OrderedSet<K> = OrderedSet(bag.take())

    /// Insert `x` if absent; returns `true` when `x` was newly inserted. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool:
        if bag.contains(x): return false
        let _ = bag.add(x)
        true

    /// Remove `x` if present; returns `true` if removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool = bag.remove(x)

    /// Whether `x` is in the set. O(log n) expected.
    pub fn contains(self, x: K) -> Bool = bag.contains(x)

    /// Smallest key in the set, or `none`. O(log n) expected.
    pub @get fn first(self) -> ?K = bag.first

    /// Largest key in the set, or `none`. O(log n) expected.
    pub @get fn last(self) -> ?K = bag.last

    /// Remove and return the smallest key, or `none`. O(log n) expected.
    pub fn pop_first(mut self) -> ?K = bag.pop_first()

    /// Remove and return the largest key, or `none`. O(log n) expected.
    pub fn pop_last(mut self) -> ?K = bag.pop_last()

    /// Smallest key `>= k`, or `none`. O(log n) expected.
    pub fn first_ge(self, k: K) -> ?K = bag.first_ge(k)

    /// Smallest key `> k`, or `none`. O(log n) expected.
    pub fn first_gt(self, k: K) -> ?K = bag.first_gt(k)

    /// Largest key `<= k`, or `none`. O(log n) expected.
    pub fn last_le(self, k: K) -> ?K = bag.last_le(k)

    /// Largest key `< k`, or `none`. O(log n) expected.
    pub fn last_lt(self, k: K) -> ?K = bag.last_lt(k)

    /// Number of keys strictly less than `k`. O(log n) expected.
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// Number of keys in `[lo, hi)`. O(log n) expected.
    pub fn count_range(self, lo: K, hi: K) -> Int = bag.count_range(lo, hi)

    /// 0-indexed `i`-th smallest key, or `none` if out of range. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// Smallest key `k` where monotone `ok(k)` holds, or `none`. O(log n) expected.
    pub fn first_where(self, ok: K -> Bool) -> ?K = bag.first_where(ok)

    /// Visit every key in ascending order. O(1) amortized per step.
    pub fn each(self) -> yields K:
        for (k, _) in bag.entries(): yield k

    /// Visit every key in descending order. O(1) amortized per step.
    pub fn each_rev(self) -> yields K:
        for (k, _) in bag.entries_rev(): yield k

    /// Return all keys in ascending order as a `List<K>`. O(n).
    pub fn to_list(self) -> List<K> = bag.to_list()

@test
fn sorted_bag_basics_duplicates_and_take():
    var b = SortedBag<Int>()
    assert b.is_empty and b.len == 0
    assert b.first == none and b.last == none
    assert b.pop_first() == none and b.pop_last() == none
    assert b.rank(5) == 0 and b.nth(0) == none and b.nth(-1) == none
    assert not b.remove(5) and not b.contains(5) and b.count(5) == 0

    for x in [5, 2, 5, 1, 2, 5, 9]: assert b.add(x)
    assert b.len == 7 and not b.is_empty
    assert b.to_list() == [1, 2, 2, 5, 5, 5, 9]
    assert b.count(5) == 3 and b.count(2) == 2 and b.count(3) == 0
    assert b.contains(2) and not b.contains(3)
    assert b.first == 1 and b.last == 9

    assert b.first_ge(2) == 2 and b.first_gt(2) == 5 and b.first_ge(10) == none and b.first_gt(9) == none
    assert b.last_le(5) == 5 and b.last_lt(5) == 2 and b.last_le(0) == none and b.last_lt(1) == none
    assert b.rank(1) == 0 and b.rank(2) == 1 and b.rank(3) == 3 and b.rank(5) == 3 and b.rank(6) == 6 and b.rank(10) == 7
    assert [b.nth(i)! for i in 0..b.len] == [1, 2, 2, 5, 5, 5, 9]
    assert b.nth(-1) == none and b.nth(7) == none

    // remove drops only one copy at a time
    assert b.remove(5) and b.count(5) == 2 and b.len == 6
    assert not b.remove(4) and b.len == 6
    assert b.pop_first() == 1 and b.pop_last() == 9
    assert b.to_list() == [2, 2, 5, 5]

    // take moves out in O(1) leaving empty; clear resets
    var moved = b.take()
    assert b.is_empty and b.len == 0 and b.first == none
    assert moved.to_list() == [2, 2, 5, 5]
    moved.clear()
    assert moved.is_empty and moved.len == 0 and moved.last == none

@test
fn sorted_bag_matches_sorted_list_on_random_ops():
    for _ in 0..8:
        var bag = SortedBag<Int>()
        var plain: List<Int> = []
        for _ in 0..120:
            let op = rand(0, 5)
            if op <= 1:
                let x = rand(-10, 10)
                assert bag.add(x)
                plain.push(x)
                sort(mut plain)
            elif op == 2:
                let x = rand(-10, 10)
                let idx = plain.find(x)
                if idx == none:
                    assert not bag.remove(x)
                else:
                    assert bag.remove(x)
                    plain.remove(idx!)
            elif op == 3:
                let got = bag.pop_first()
                if plain.is_empty:
                    assert got == none
                else:
                    assert got == plain.first
                    plain.remove(0)
            elif op == 4:
                let got = bag.pop_last()
                if plain.is_empty:
                    assert got == none
                else:
                    assert got == plain.pop()
            else:
                let q = rand(-12, 12)
                assert bag.len == plain.len and bag.is_empty == plain.is_empty
                assert bag.first == plain.first and bag.last == plain.last
                assert bag.count(q) == count(x == q for x in plain)
                assert bag.contains(q) == (q in plain)
                assert bag.first_ge(q) == first(x for x in plain if x >= q)
                assert bag.first_gt(q) == first(x for x in plain if x > q)
                assert bag.last_le(q) == max(x for x in plain if x <= q)
                assert bag.last_lt(q) == max(x for x in plain if x < q)
                assert bag.rank(q) == count(x < q for x in plain)
                if not plain.is_empty:
                    let i = rand(0, plain.len - 1)
                    assert bag.nth(i) == plain[i]
                assert bag.nth(plain.len) == none
        assert bag.to_list() == plain

@test
fn sorted_bag_inversion_count_and_generic_keys():
    // Mirrors noi22p5: count inversions over a domain using rank(), plus of() over strings
    domain Pos = 1..=6
    let a = [Pos] i => [4, 2, 4, 1, 3, 2][i.to_idx]
    var seen = SortedBag<Int>()
    var inv = 0
    for i in (Pos.first..=Pos.last).rev:
        inv += seen.rank(a[i])
        seen.add(a[i])
    var brute_inv = 0
    for i in Pos:
        for j in Pos:
            if i < j and a[i] > a[j]: brute_inv += 1
    assert inv == brute_inv

    let words = SortedBag<Str>.of(["pear", "fig", "apple", "fig", "kiwi"])
    assert words.len == 5 and words.count("fig") == 2
    assert words.first == "apple" and words.last == "pear"
    assert words.first_gt("fig") == "kiwi" and words.last_lt("kiwi") == "fig"
    assert words.rank("fig") == 1 and words.nth(2) == "fig"

@test
fn sorted_bag_cursors_entries_and_recycling():
    var b = SortedBag<Int>()
    b.add_many(10, 3)
    b.add_many(20, 1)
    b.add_many(30, 2)
    b.add_many(15, 4)
    assert b.len == 10 and b.distinct == 4
    assert b.count_range(10, 25) == 8
    assert b.count_range(25, 10) == 0
    assert b.first_where(x => x >= 16) == 20
    assert b.first_where(x => x > 100) == none

    var fwd: List<(Int, Int)> = []
    for (k, c) in b.entries(): fwd.push((k, c))
    assert fwd == [(10, 3), (15, 4), (20, 1), (30, 2)]

    var rev: List<(Int, Int)> = []
    for (k, c) in b.entries_rev(): rev.push((k, c))
    assert rev == [(30, 2), (20, 1), (15, 4), (10, 3)]

    // Cursor stepping matches sorted distinct keys in both directions.
    let c20 = b.cursor_at(20)!
    assert b.key_of(c20) == 20 and b.count_of(c20) == 1
    assert b.key_of(b.prev_cursor(c20)!) == 15
    assert b.key_of(b.next_cursor(c20)!) == 30
    assert b.next_cursor(b.cursor_last()!) == none
    assert b.prev_cursor(b.cursor_first()!) == none

    // Recycling keeps internal slot count bounded under heavy insert/remove churn.
    assert b.remove_all(15) == 4
    assert b.remove_all(99) == 0
    assert b.distinct == 3
    for i in 0..200:
        let _ = b.add(1000 + i)
        assert b.remove(1000 + i)
    assert b.distinct == 3
    assert b.key.len <= 10

@test
fn ordered_set_operations_and_first_where():
    var s = OrderedSet<Int>.of([5, 1, 5, 3, 9, 3])
    assert s.len == 4 and not s.is_empty
    assert not s.add(3) and s.add(7)
    assert s.to_list() == [1, 3, 5, 7, 9]
    assert s.rank(5) == 2 and s.count_range(3, 8) == 3
    assert s.nth(2) == 5
    assert s.first_where(x => x * x >= 25) == 5
    var rev: List<Int> = []
    for x in s.each_rev(): rev.push(x)
    assert rev == [9, 7, 5, 3, 1]
    assert s.remove(5) and not s.remove(5)
    assert s.len == 4
