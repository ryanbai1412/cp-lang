//! Ordered multiset with order statistics: `SortedBag<K>`.

/// An ordered multiset (bag) of `K` backed by a randomized treap over flat arrays.
/// Keeps one node per distinct key with its multiplicity and subtree element count.
/// `add`, `remove`, `contains`, `count`, `first`, `last`, `pop_first`, `pop_last`,
/// `first_ge`, `first_gt`, `last_le`, `last_lt`, `rank`, and `nth` are O(log n) expected;
/// `len`, `is_empty`, `clear`, and `take` are O(1); `to_list` is O(n).
pub struct SortedBag<K>:
    key: List<K>
    pri: List<Int>
    cnt: List<Int>
    sub: List<Int>
    left: List<?Int>
    right: List<?Int>
    root: ?Int

    pub fn new() -> SortedBag<K> = SortedBag([], [], [], [], [], [], none)

    /// Build a `SortedBag` containing every element of `xs`. O(n log n) expected.
    pub fn of(xs: List<K>) -> SortedBag<K>:
        var b = SortedBag<K>.new()
        for x in xs: b.add(x)
        b

    /// Total number of elements in the bag (counting duplicates). O(1).
    pub @get fn len(self) -> Int = self.sub_of(root)

    pub @get fn is_empty(self) -> Bool = self.len == 0

    pub fn clear(mut self):
        key = []
        pri = []
        cnt = []
        sub = []
        left = []
        right = []
        root = none

    /// Move the bag's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> SortedBag<K>:
        let out = SortedBag(key, pri, cnt, sub, left, right, root)
        key = []
        pri = []
        cnt = []
        sub = []
        left = []
        right = []
        root = none
        out

    /// Insert one copy of `x` into the bag. Always returns `true`. O(log n) expected.
    pub fn add(mut self, x: K) -> Bool:
        let (lo, rest) = self.split_lt(root, x)
        let (eq, hi) = self.split_le(rest, x)
        var mid = eq
        if mid != none:
            let u = mid!
            cnt[u] += 1
            self.pull(u)
        else:
            mid = self.make_node(x)
        let lm = self.merge(lo, mid)
        root = self.merge(lm, hi)
        true

    /// Remove one copy of `x` if present; returns `true` if a copy was removed. O(log n) expected.
    pub fn remove(mut self, x: K) -> Bool:
        let (lo, rest) = self.split_lt(root, x)
        let (eq, hi) = self.split_le(rest, x)
        if eq == none:
            root = self.merge(lo, hi)
            return false
        let u = eq!
        var mid: ?Int = none
        if cnt[u] > 1:
            cnt[u] -= 1
            self.pull(u)
            mid = u
        let lm = self.merge(lo, mid)
        root = self.merge(lm, hi)
        true

    /// How many copies of `x` are currently in the bag. O(log n) expected.
    pub fn count(self, x: K) -> Int:
        var at = root
        while at != none:
            let u = at!
            if x < key[u]: at = left[u]
            elif key[u] < x: at = right[u]
            else: return cnt[u]
        0

    /// Whether at least one copy of `x` is in the bag. O(log n) expected.
    pub fn contains(self, x: K) -> Bool = self.count(x) > 0

    /// The smallest element, or `none` when empty. O(log n) expected.
    pub @get fn first(self) -> ?K:
        if root == none: return none
        var u = root!
        while left[u] != none: u = left[u]!
        key[u]

    /// The largest element, or `none` when empty. O(log n) expected.
    pub @get fn last(self) -> ?K:
        if root == none: return none
        var u = root!
        while right[u] != none: u = right[u]!
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
        var at = root
        while at != none:
            let u = at!
            if not (key[u] < k):
                ans = key[u]
                at = left[u]
            else:
                at = right[u]
        ans

    /// Smallest element `> k`, or `none`. O(log n) expected.
    pub fn first_gt(self, k: K) -> ?K:
        var ans: ?K = none
        var at = root
        while at != none:
            let u = at!
            if k < key[u]:
                ans = key[u]
                at = left[u]
            else:
                at = right[u]
        ans

    /// Largest element `<= k`, or `none`. O(log n) expected.
    pub fn last_le(self, k: K) -> ?K:
        var ans: ?K = none
        var at = root
        while at != none:
            let u = at!
            if not (k < key[u]):
                ans = key[u]
                at = right[u]
            else:
                at = left[u]
        ans

    /// Largest element `< k`, or `none`. O(log n) expected.
    pub fn last_lt(self, k: K) -> ?K:
        var ans: ?K = none
        var at = root
        while at != none:
            let u = at!
            if key[u] < k:
                ans = key[u]
                at = right[u]
            else:
                at = left[u]
        ans

    /// Number of elements strictly less than `k` (counting duplicates). O(log n) expected.
    pub fn rank(self, k: K) -> Int:
        var ans = 0
        var at = root
        while at != none:
            let u = at!
            if key[u] < k:
                ans += self.sub_of(left[u]) + cnt[u]
                at = right[u]
            else:
                at = left[u]
        ans

    /// 0-indexed `i`-th smallest element (counting duplicates), or `none` if out of bounds. O(log n) expected.
    pub fn nth(self, i: Int) -> ?K:
        if i < 0 or i >= self.len: return none
        var k = i
        var at = root
        while at != none:
            let u = at!
            let lsz = self.sub_of(left[u])
            if k < lsz:
                at = left[u]
            elif k < lsz + cnt[u]:
                return key[u]
            else:
                k -= lsz + cnt[u]
                at = right[u]
        none

    /// Every element in ascending order, repeated by multiplicity. O(n).
    pub fn to_list(self) -> List<K>:
        var out: List<K> = []
        var stack: List<Int> = []
        var at = root
        while at != none or not stack.is_empty:
            while at != none:
                stack.push(at!)
                at = left[at!]
            let u = stack.pop()!
            for _ in 0..cnt[u]: out.push(key[u])
            at = right[u]
        out

    fn sub_of(self, t: ?Int) -> Int = if t == none: 0 else: sub[t!]

    fn pull(mut self, u: Int):
        sub[u] = cnt[u] + self.sub_of(left[u]) + self.sub_of(right[u])

    fn make_node(mut self, x: K) -> Int:
        key.push(x)
        pri.push(rand(0, 1 << 60))
        cnt.push(1)
        sub.push(1)
        left.push(none)
        right.push(none)
        key.len - 1

    /// Split `t` into `(keys < x, keys >= x)`.
    fn split_lt(mut self, t: ?Int, x: K) -> (?Int, ?Int):
        if t == none: return (none, none)
        let u = t!
        if key[u] < x:
            let (a, b) = self.split_lt(right[u], x)
            right[u] = a
            self.pull(u)
            (u, b)
        else:
            let (a, b) = self.split_lt(left[u], x)
            left[u] = b
            self.pull(u)
            (a, u)

    /// Split `t` into `(keys <= x, keys > x)`.
    fn split_le(mut self, t: ?Int, x: K) -> (?Int, ?Int):
        if t == none: return (none, none)
        let u = t!
        if not (x < key[u]):
            let (a, b) = self.split_le(right[u], x)
            right[u] = a
            self.pull(u)
            (u, b)
        else:
            let (a, b) = self.split_le(left[u], x)
            left[u] = b
            self.pull(u)
            (a, u)

    fn merge(mut self, a: ?Int, b: ?Int) -> ?Int:
        if a == none: return b
        if b == none: return a
        let (x, y) = (a!, b!)
        if pri[x] >= pri[y]:
            let m = self.merge(right[x], y)
            right[x] = m
            self.pull(x)
            x
        else:
            let m = self.merge(x, left[y])
            left[y] = m
            self.pull(y)
            y

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
                    plain = [plain[j] for j in 0..plain.len if j != idx!]
            elif op == 3:
                let got = bag.pop_first()
                if plain.is_empty:
                    assert got == none
                else:
                    assert got == plain[0]
                    plain = [plain[j] for j in 1..plain.len]
            elif op == 4:
                let got = bag.pop_last()
                if plain.is_empty:
                    assert got == none
                else:
                    assert got == plain[plain.len - 1]
                    plain.pop()
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
