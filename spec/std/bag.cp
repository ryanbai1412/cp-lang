//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`.

use std.seq.{remove}

/// Read-only handle into a `SortedBag<K>` node, valid until the next mutation.
pub struct BagCursor(id: Int)

struct BagNode<K>:
    key: K
    cnt: Int
    sub: Int
    pri: -2_147_483_648..=2_147_483_647
    left: -2_147_483_648..=2_147_483_647
    right: -2_147_483_648..=2_147_483_647
    par: -2_147_483_648..=2_147_483_647

/// An ordered multiset (bag) of `K` backed by a randomized treap over a single flat node array
/// with recycled node slots and parent pointers for O(1) amortized cursor stepping.
/// Keeps one node per distinct key with its multiplicity and subtree element count.
/// `add`, `add_many`, `remove`, `remove_all`, `contains`, `count`, `first`, `last`,
/// `pop_first`, `pop_last`, `rank`, `nth`, `bsearch_first`, `bsearch_last`, and `cursor_*` are O(log n) expected;
/// `range(r)` returns an O(1) view whose `.first`, `.last`, `.len`, `.is_empty` are O(log n)
/// expected and whose iteration (`for x in b.range(r)` / `.rev`) is O(log n + k);
/// `key_of` and `count_of` are O(1); `next_cursor` and `prev_cursor` are O(1) amortized;
/// `len`, `distinct`, `is_empty`, `clear`, and `take` are O(1);
/// `entries`, `entries_rev`, and `to_list` are O(n).
pub struct SortedBag<K>:
    nodes: List<BagNode<K>>
    free: List<-2_147_483_648..=2_147_483_647>
    live: Int
    root: Int
    rng: U64

    /// Create an empty `SortedBag<K>`. O(1).
    pub fn new() -> SortedBag<K> = SortedBag([], [], 0, -1, U64(0x5e3779b97f4a7c15))

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
        nodes = []
        free = []
        live = 0
        root = -1

    /// Move the bag's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> SortedBag<K>:
        let out = SortedBag(nodes, free, live, root, rng)
        nodes = []
        free = []
        live = 0
        root = -1
        out

    /// Insert one copy of `x` into the bag. Always returns `true`. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool:
        let (r, _) = self.insert_at(root, x, 1, false)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        true

    fn add_unique(mut self, x: K) -> Bool:
        let (r, added) = self.insert_at(root, x, 1, true)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        added

    /// Insert `c >= 0` copies of `x` into the bag. O(log n) expected.
    pub fn add_many(mut self, x: K, c: Int):
        assert c >= 0
        if c == 0: return
        let (r, _) = self.insert_at(root, x, c, false)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1

    /// Remove one copy of `x` if present; returns `true` if a copy was removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool:
        let (r, removed) = self.erase_at(root, x, false)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        removed > 0

    /// Remove all copies of `x` and return how many were removed. O(log n) expected.
    pub fn remove_all(mut self, x: K) -> Int:
        let (r, removed) = self.erase_at(root, x, true)
        root = r
        unchecked:
            if root >= 0: nodes[root].par = -1
        removed

    /// How many copies of `x` are currently in the bag. O(log n) expected.
    pub fn count(self, x: K) -> Int:
        var u = root
        unchecked:
            while u >= 0:
                if x < nodes[u].key: u = nodes[u].left
                elif nodes[u].key < x: u = nodes[u].right
                else: return nodes[u].cnt
        0

    /// Whether at least one copy of `x` is in the bag. O(log n) expected.
    pub fn contains(self, x: K) -> Bool = self.count(x) > 0

    /// The smallest element, or `none` when empty. O(log n) expected.
    pub @get fn first(self) -> ?K:
        if root < 0: return none
        var u = root
        unchecked:
            while nodes[u].left >= 0: u = nodes[u].left
            return nodes[u].key

    /// The largest element, or `none` when empty. O(log n) expected.
    pub @get fn last(self) -> ?K:
        if root < 0: return none
        var u = root
        unchecked:
            while nodes[u].right >= 0: u = nodes[u].right
            return nodes[u].key

    /// Remove and return one copy of the smallest element, or `none` when empty. O(log n) expected.
    pub fn pop_first(mut self) -> ?K:
        let x = self.first
        if x == none: return none
        let _ = self.remove(x!)
        x

    /// Remove and return one copy of the largest element, or `none` when empty. O(log n) expected.
    pub fn pop_last(mut self) -> ?K:
        let x = self.last
        if x == none: return none
        let _ = self.remove(x!)
        x

    /// Number of elements strictly less than `k` (counting duplicates). O(log n) expected.
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
            if ans < 0: return none
            let k = nodes[ans].key
            return if self.hi_ok(k, hi, hi_inc): k else: none

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
            if ans < 0: return none
            let k = nodes[ans].key
            return if self.lo_ok(k, lo, lo_inc): k else: none

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

    /// 0-indexed `i`-th smallest element (counting duplicates), or `none` if out of bounds. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K:
        if i < 0 or i >= self.len: return none
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
        none

    /// Smallest key where monotone `ok(k)` holds (`false..false, true..true`), or `none`.
    /// Checks monotonicity at the boundary. O(log n) expected.
    pub fn bsearch_first(self, ok: K -> Bool) -> ?K:
        var ans = -1
        var u = root
        unchecked:
            while u >= 0:
                if ok(nodes[u].key):
                    ans = u
                    u = nodes[u].left
                else:
                    u = nodes[u].right
        if ans < 0:
            if let lo = self.first: assert not ok(lo)
            if let hi = self.last: assert not ok(hi)
            return none
        if let prev = self.prev_cursor(BagCursor(ans)):
            assert not ok(nodes[prev.id].key)
        nodes[ans].key

    /// Largest key where monotone `ok(k)` holds (`true..true, false..false`), or `none`.
    /// Checks monotonicity at the boundary. O(log n) expected.
    pub fn bsearch_last(self, ok: K -> Bool) -> ?K:
        var ans = -1
        var u = root
        unchecked:
            while u >= 0:
                if ok(nodes[u].key):
                    ans = u
                    u = nodes[u].right
                else:
                    u = nodes[u].left
        if ans < 0:
            if let lo = self.first: assert not ok(lo)
            if let hi = self.last: assert not ok(hi)
            return none
        if let next = self.next_cursor(BagCursor(ans)):
            assert not ok(nodes[next.id].key)
        nodes[ans].key

    /// Remove and return the maximal run of elements strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest element (ascending)
    /// while `ok(k)` holds. O(k log n) expected for `k` removed elements.
    pub fn drain(mut self; after: ?K = none, before: ?K = none, while ok: K -> Bool) -> List<K>:
        assert not (
            after != none and before != none
        ), "drain takes either after: or before:, not both"
        var out: List<K> = []
        if let b = before:
            while let x = self.range_last(none, false, some(b), false):
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        elif let a = after:
            while let x = self.range_first(some(a), false, none, false):
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        else:
            while let x = self.first:
                if not ok(x): break
                let _ = self.remove(x)
                out.push(x)
        out

    /// Cursor to the smallest key, or `none` when empty. O(log n) expected.
    pub fn cursor_first(self) -> ?BagCursor:
        if root < 0: return none
        var u = root
        unchecked:
            while nodes[u].left >= 0: u = nodes[u].left
        BagCursor(u)

    /// Cursor to the largest key, or `none` when empty. O(log n) expected.
    pub fn cursor_last(self) -> ?BagCursor:
        if root < 0: return none
        var u = root
        unchecked:
            while nodes[u].right >= 0: u = nodes[u].right
        BagCursor(u)

    /// Cursor to exact key `k`, or `none` if absent. O(log n) expected.
    pub fn cursor_at(self, k: K) -> ?BagCursor:
        var u = root
        unchecked:
            while u >= 0:
                if k < nodes[u].key: u = nodes[u].left
                elif nodes[u].key < k: u = nodes[u].right
                else: return BagCursor(u)
        none

    /// Cursor to smallest key `>= k`, or `none`. O(log n) expected.
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
        if ans >= 0: BagCursor(ans) else: none

    /// Cursor to smallest key `> k`, or `none`. O(log n) expected.
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
        if ans >= 0: BagCursor(ans) else: none

    /// Cursor to largest key `<= k`, or `none`. O(log n) expected.
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
        if ans >= 0: BagCursor(ans) else: none

    /// Cursor to largest key `< k`, or `none`. O(log n) expected.
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
        if ans >= 0: BagCursor(ans) else: none

    /// Key stored at cursor `c`. O(1).
    pub fn key_of(self, c: BagCursor) -> K = nodes[c.id].key

    /// Multiplicity stored at cursor `c`. O(1).
    pub fn count_of(self, c: BagCursor) -> Int = nodes[c.id].cnt

    /// Cursor to the next larger distinct key, or `none`. O(1) amortized.
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
        none

    /// Cursor to the next smaller distinct key, or `none`. O(1) amortized.
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
        none

    /// Visit every distinct `(key, count)` in ascending order. O(1) amortized per step.
    pub fn entries(self) -> yields (key: K, count: Int):
        var cur = self.cursor_first()
        while let c = cur:
            yield (key: nodes[c.id].key, count: nodes[c.id].cnt)
            cur = self.next_cursor(c)

    /// Visit every distinct `(key, count)` in descending order. O(1) amortized per step.
    pub fn entries_rev(self) -> yields (key: K, count: Int):
        var cur = self.cursor_last()
        while let c = cur:
            yield (key: nodes[c.id].key, count: nodes[c.id].cnt)
            cur = self.prev_cursor(c)

    /// Every element in ascending order, repeated by multiplicity. O(n).
    pub fn to_list(self) -> List<K>:
        var out: List<K> = []
        for (k, c) in self.entries():
            for _ in 0..c: out.push(k)
        out

    /// Preallocate treap node pool for at least `cap` distinct keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Split off and return all elements `>= k`, keeping `< k` in `self`. O(log n).
    pub fn split_at(mut self, k: K) -> SortedBag<K> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    pub fn join(mut self, other: SortedBag<K>) = todo

    /// Merge all elements of `other` into `self` (small-to-large, emptying `other`). O(min(n, m) log(n + m)).
    pub fn absorb(mut self, mut other: SortedBag<K>) = todo

    /// Sum of all elements (for additive `K`). O(1) when augmented, O(n) otherwise.
    pub fn sum(self) -> Int = todo

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

/// An ordered set of unique keys `K` with order statistics (`rank`, `nth`),
/// O(1) key-range views (`range(r)`), and monotone search (`bsearch_first`/`bsearch_last`). Backed by `SortedBag<K>`.
/// All single-key and range-summary operations are O(log n) expected;
/// `range(r)` is O(1); range iteration is O(log n + k);
/// `len`, `is_empty`, `clear`, `take` are O(1); `each`, `each_rev`, and `to_list` are O(n).
pub struct SortedSet<K>:
    bag: SortedBag<K>

    /// Create an empty `SortedSet<K>`. O(1).
    pub fn new() -> SortedSet<K> = SortedSet(SortedBag<K>.new())

    /// Build a `SortedSet` containing the distinct elements of `xs`. O(n log n) expected.
    pub fn of(xs: List<K>) -> SortedSet<K>:
        var s = SortedSet<K>.new()
        for x in xs: let _ = s.add(x)
        s

    /// Number of keys in the set. O(1).
    pub @get fn len(self) -> Int = bag.len

    /// Whether the set is empty. O(1).
    pub @get fn is_empty(self) -> Bool = bag.is_empty

    /// Remove all keys from the set. O(1).
    pub fn clear(mut self): bag.clear()

    /// Move all keys out, leaving `self` empty. O(1).
    pub fn take(mut self) -> SortedSet<K> = SortedSet(bag.take())

    /// Insert `x` if absent; returns `true` when `x` was newly inserted. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool = bag.add_unique(x)

    /// Remove `x` if present; returns `true` if removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool = bag.remove(x)

    /// Add `x` when `on` is true, or remove it otherwise, matching `Set.set` and `Bitset.set`. O(log n) expected.
    pub fn set(mut self, x: K, on: Bool):
        if on:
            let _ = self.add(x)
        else:
            let _ = self.remove(x)

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

    /// Number of keys strictly less than `k`. O(log n) expected.
    pub fn rank(self, k: K) -> Int = bag.rank(k)

    /// 0-indexed `i`-th smallest key, or `none` if out of range. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K = bag.nth(i)

    /// Smallest key `k` where monotone `ok(k)` holds (`false..false, true..true`), or `none`. O(log n) expected.
    pub fn bsearch_first(self, ok: K -> Bool) -> ?K = bag.bsearch_first(ok)

    /// Largest key `k` where monotone `ok(k)` holds (`true..true, false..false`), or `none`. O(log n) expected.
    pub fn bsearch_last(self, ok: K -> Bool) -> ?K = bag.bsearch_last(ok)

    /// Remove and return the maximal run of keys strictly after `after` (ascending),
    /// strictly before `before` (descending), or from the smallest key (ascending)
    /// while `ok(k)` holds. O(k log n) expected for `k` removed keys.
    pub fn drain(mut self; after: ?K = none, before: ?K = none, while ok: K -> Bool) -> List<K> =
        bag.drain(after: after, before: before, while: ok)

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K =
        bag.range_first(lo, lo_inc, hi, hi_inc)

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K =
        bag.range_last(lo, lo_inc, hi, hi_inc)

    fn range_len(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> Int =
        bag.range_len(lo, lo_inc, hi, hi_inc)

    fn range_each(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        for k in bag.range_each(lo, lo_inc, hi, hi_inc): yield k

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        for k in bag.range_each_rev(lo, lo_inc, hi, hi_inc): yield k

    /// Visit every key in ascending order. O(1) amortized per step.
    pub fn each(self) -> yields K:
        for (k, _) in bag.entries(): yield k

    /// Visit every key in descending order. O(1) amortized per step.
    pub fn each_rev(self) -> yields K:
        for (k, _) in bag.entries_rev(): yield k

    /// Return all keys in ascending order as a `List<K>`. O(n).
    pub fn to_list(self) -> List<K> = bag.to_list()

    /// Preallocate treap node pool for at least `cap` keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Split off and return all keys `>= k`, keeping `< k` in `self`. O(log n).
    pub fn split_at(mut self, k: K) -> SortedSet<K> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    pub fn join(mut self, other: SortedSet<K>) = todo

    /// Merge all keys of `other` into `self` (small-to-large, emptying `other`). O(min(n, m) log(n + m)).
    pub fn absorb(mut self, mut other: SortedSet<K>) = todo

/// Order-statistic ordered map from `K` to `V` backed by a randomized treap.
/// `get`, `set`, `remove`, `rank`, `nth`, `first_ge`/`last_le`, `bsearch_first`/`bsearch_last`, `split_at`, `join` are O(log n).
pub struct SortedMap<K, V>:
    _len: Int

    /// Create an empty `SortedMap<K, V>`. O(1).
    pub fn new() -> SortedMap<K, V> = todo

    /// Preallocate treap node pool for at least `cap` entries. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of key-value pairs. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the map is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all entries. O(1).
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> SortedMap<K, V> = todo

    /// Look up `k`, returning `none` if absent. O(log n).
    pub fn get(self, k: K) -> ?V = todo

    /// Whether `k` is present. O(log n).
    pub fn contains(self, k: K) -> Bool = todo

    /// Insert or overwrite `k -> v` (W8 `IndexSet` backing `m[k] = v`). O(log n).
    pub fn set(mut self, k: K, v: V) = todo

    /// Remove `k` and return its previous value, or `none` if absent. O(log n).
    pub fn remove(mut self, k: K) -> ?V = todo

    /// Smallest `(key, val)` pair, or `none` if empty. O(log n).
    pub @get fn first(self) -> ?(K, V) = todo

    /// Largest `(key, val)` pair, or `none` if empty. O(log n).
    pub @get fn last(self) -> ?(K, V) = todo

    /// Remove and return the smallest `(key, val)` pair, or `none`. O(log n).
    pub fn pop_first(mut self) -> ?(K, V) = todo

    /// Remove and return the largest `(key, val)` pair, or `none`. O(log n).
    pub fn pop_last(mut self) -> ?(K, V) = todo

    /// Smallest entry with key `>= k`, or `none`. O(log n).
    pub fn first_ge(self, k: K) -> ?(K, V) = todo

    /// Smallest entry with key `> k`, or `none`. O(log n).
    pub fn first_gt(self, k: K) -> ?(K, V) = todo

    /// Largest entry with key `<= k`, or `none`. O(log n).
    pub fn last_le(self, k: K) -> ?(K, V) = todo

    /// Largest entry with key `< k`, or `none`. O(log n).
    pub fn last_lt(self, k: K) -> ?(K, V) = todo

    /// Number of entries with key strictly less than `k`. O(log n).
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th entry in key order (0-based), or `none`. O(log n).
    pub fn nth(self, i: Int) -> ?(K, V) = todo

    /// Smallest entry where monotone `ok(k, v)` becomes true, or `none`. O(log n).
    pub fn bsearch_first(self, ok: (K, V) -> Bool) -> ?(K, V) = todo

    /// Largest entry where monotone `ok(k, v)` still holds, or `none`. O(log n).
    pub fn bsearch_last(self, ok: (K, V) -> Bool) -> ?(K, V) = todo

    /// Remove and return entries after `after` (ascending) or before `before` (descending) while `ok` holds. O(k log n).
    pub fn drain(mut self; after: ?K = none, before: ?K = none, while ok: (K, V) -> Bool) -> List<(K, V)> = todo

    /// Split off and return all entries with key `>= k`. O(log n).
    pub fn split_at(mut self, k: K) -> SortedMap<K, V> = todo

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`. O(log n).
    pub fn join(mut self, other: SortedMap<K, V>) = todo

    /// Merge all entries of `other` into `self` (emptying `other`), combining collisions with `join` or overwriting. O(min(n, m) log(n + m)).
    pub fn absorb(mut self, mut other: SortedMap<K, V>, join: ?((V, V) -> V) = none) = todo

    /// Yield `(key, val)` pairs in ascending key order. O(n).
    pub fn each(self) -> yields (K, V) = todo

    /// Yield `(key, val)` pairs in descending key order. O(n).
    pub fn each_rev(self) -> yields (K, V) = todo

    /// Yield keys in ascending order. O(n).
    pub fn keys(self) -> yields K = todo

    /// Yield values in ascending key order. O(n).
    pub fn values(self) -> yields V = todo

/// Monoid-augmented ordered multiset maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Supports O(log n) `add`, `remove`, `rank`, `nth`, `fold_lt`, `fold_le`, `fold_ge`, `fold_gt`, `fold_between`, and O(1) `all`.
pub struct AugSortedBag<K, T>:
    _len: Int

    /// Create an empty `AugSortedBag<K, T>` with associative `op`, identity `id`, and key map `leaf`. O(1).
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedBag<K, T> = todo

    /// Preallocate treap node pool for at least `cap` elements. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Total number of elements (with multiplicity). O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the bag is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Fold of all elements in the bag. O(1).
    pub @get fn all(self) -> T = todo

    /// Insert one copy of `x`. O(log n).
    pub fn add(mut self, x: K) = todo

    /// Remove one copy of `x`; returns false if absent. O(log n).
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Multiplicity of `x`. O(log n).
    pub fn count(self, x: K) -> Int = todo

    /// Number of elements strictly less than `k`. O(log n).
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th smallest element (0-based), or `none`. O(log n).
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest element, or `none`. O(log n).
    pub @get fn first(self) -> ?K = todo

    /// Largest element, or `none`. O(log n).
    pub @get fn last(self) -> ?K = todo

    /// Fold elements with key `< hi`. O(log n).
    pub fn fold_lt(self, hi: K) -> T = todo

    /// Fold elements with key `<= hi`. O(log n).
    pub fn fold_le(self, hi: K) -> T = todo

    /// Fold elements with key `>= lo`. O(log n).
    pub fn fold_ge(self, lo: K) -> T = todo

    /// Fold elements with key `> lo`. O(log n).
    pub fn fold_gt(self, lo: K) -> T = todo

    /// Fold elements with key in `[lo, hi)`. O(log n).
    pub fn fold_between(self, lo: K, hi: K) -> T = todo

    /// Yield all elements in ascending order. O(n).
    pub fn each(self) -> yields K = todo

/// Monoid-augmented ordered unique set maintaining `op` over `leaf(k)` on every subtree (W10 operation values).
/// Covers dynamic range sums and consecutive-gap monoids (`cf2066e`) in O(log n).
pub struct AugSortedSet<K, T>:
    _len: Int

    /// Create an empty `AugSortedSet<K, T>` with associative `op`, identity `id`, and key map `leaf`. O(1).
    pub fn new(op: (T, T) -> T, id: T, leaf: K -> T) -> AugSortedSet<K, T> = todo

    /// Preallocate treap node pool for at least `cap` keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of distinct keys. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the set is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Fold of all keys in the set. O(1).
    pub @get fn all(self) -> T = todo

    /// Insert `x`; returns true if `x` was new. O(log n).
    pub fn add(mut self, x: K) -> Bool = todo

    /// Remove `x`; returns true if `x` was present. O(log n).
    pub fn remove(mut self, x: K) -> Bool = todo

    /// Whether `x` is present. O(log n).
    pub fn contains(self, x: K) -> Bool = todo

    /// Number of keys strictly less than `k`. O(log n).
    pub fn rank(self, k: K) -> Int = todo

    /// The `i`-th smallest key (0-based), or `none`. O(log n).
    pub fn nth(self, i: Int) -> ?K = todo

    /// Smallest key, or `none`. O(log n).
    pub @get fn first(self) -> ?K = todo

    /// Largest key, or `none`. O(log n).
    pub @get fn last(self) -> ?K = todo

    /// Fold keys `< hi`. O(log n).
    pub fn fold_lt(self, hi: K) -> T = todo

    /// Fold keys `<= hi`. O(log n).
    pub fn fold_le(self, hi: K) -> T = todo

    /// Fold keys `>= lo`. O(log n).
    pub fn fold_ge(self, lo: K) -> T = todo

    /// Fold keys `> lo`. O(log n).
    pub fn fold_gt(self, lo: K) -> T = todo

    /// Fold keys in `[lo, hi)`. O(log n).
    pub fn fold_between(self, lo: K, hi: K) -> T = todo

    /// Yield all keys in ascending order. O(n).
    pub fn each(self) -> yields K = todo

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

    assert b.range(2..).first == 2 and b.range(3..).first == 5 and b.range(10..).first == none and b.range(10..).first == none
    assert b.range(..=5).last == 5 and b.range(..5).last == 2 and b.range(..=0).last == none and b.range(..1).last == none
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
                assert bag.range(q..).first == first(x for x in plain if x >= q)
                assert bag.range(q + 1..).first == first(x for x in plain if x > q)
                assert bag.range(..=q).last == max(x for x in plain if x <= q)
                assert bag.range(..q).last == max(x for x in plain if x < q)
                let lo = rand(-12, 12)
                let hi = rand(-12, 12)
                let slice = [x for x in plain if lo <= x and x < hi]
                let v = bag.range(lo..hi)
                assert v.len == slice.len and v.is_empty == slice.is_empty
                assert v.first == slice.first and v.last == slice.last
                assert [x for x in v] == slice
                assert [x for x in v.rev] == slice.rev
                let inc = [x for x in plain if lo <= x and x <= hi]
                assert bag.range(lo..=hi).len == inc.len
                assert bag.range(lo..=hi).first == inc.first
                assert bag.range(lo..=hi).last == inc.last
                assert [x for x in bag.range(lo..=hi)] == inc
                let (before, after) = (bag.range(..q), bag.range(q..))
                assert before.len + after.len == plain.len
                assert before.last == max(x for x in plain if x < q)
                assert after.first == first(x for x in plain if x >= q)
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
    assert words.bsearch_first(w => w > "fig") == "kiwi" and words.range(.."kiwi").last == "fig"
    assert words.bsearch_last(w => w <= "fig") == "fig"
    assert words.rank("fig") == 1 and words.nth(2) == "fig"

@test
fn sorted_bag_cursors_entries_and_recycling():
    var b = SortedBag<Int>()
    b.add_many(10, 3)
    b.add_many(20, 1)
    b.add_many(30, 2)
    b.add_many(15, 4)
    assert b.len == 10 and b.distinct == 4
    assert b.range(10..25).len == 8
    assert b.range(25..10).len == 0
    assert b.bsearch_first(x => x >= 16) == 20
    assert b.bsearch_first(x => x > 100) == none
    assert b.bsearch_last(x => x <= 16) == 15
    assert b.bsearch_last(x => x < 10) == none

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
    assert b.nodes.len <= 10

@test
fn sorted_set_operations_range_and_bsearch():
    var s = SortedSet<Int>.of([5, 1, 5, 3, 9, 3])
    assert s.len == 4 and not s.is_empty
    assert not s.add(3) and s.add(7)
    assert s.to_list() == [1, 3, 5, 7, 9]
    assert s.rank(5) == 2 and s.range(3..8).len == 3
    assert s.nth(2) == 5
    assert s.bsearch_first(x => x * x >= 25) == 5
    assert s.bsearch_last(x => x * x <= 30) == 5
    s.set(6, on: true)
    assert s.contains(6) and s.len == 6
    s.set(6, on: true)
    assert s.contains(6) and s.len == 6
    s.set(6, on: false)
    assert not s.contains(6) and s.len == 5
    s.set(6, on: false)
    assert not s.contains(6) and s.len == 5
    var rev: List<Int> = []
    for x in s.each_rev(): rev.push(x)
    assert rev == [9, 7, 5, 3, 1]
    assert [x for x in s.range(3..=7)] == [3, 5, 7]
    assert [x for x in s.range(3..=7).rev] == [7, 5, 3]
    assert s.remove(5) and not s.remove(5)
    assert s.len == 4

    // Domain-keyed SortedSet with Range<V> and open Bounds (..r.start, r.end..), cf2062e2 shape
    domain V = 0..8
    var ds = SortedSet<V>()
    for i in [1, 3, 5, 6]:
        let _ = ds.add(V.nth(i))
    let r = V.nth(2)..V.nth(6)
    let (before, after) = (ds.range(..r.start), ds.range(r.end..))
    assert before.len == 1 and before.first == V.nth(1) and before.last == V.nth(1)
    assert after.len == 1 and after.first == V.nth(6) and after.last == V.nth(6)
    assert ds.range(r).len == 2 and ds.range(r).first == V.nth(3) and ds.range(r).last == V.nth(5)
    assert [x.to_idx for x in ds.range(r)] == [3, 5]
    assert [x.to_idx for x in ds.range(r).rev] == [5, 3]
    let full = ds.range(..)
    assert full.len == 4 and full.first == V.nth(1) and full.last == V.nth(6)

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
