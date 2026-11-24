//! Ordered multiset and set with order statistics, range views, and cursors: `SortedBag<K>`, `SortedSet<K>`.

use std.seq.Track

/// Read-only handle to one distinct key of a `SortedBag<K>`, valid until the next mutation.
pub struct BagCursor(id: Int)

// Slots per B+tree node; a node other than the root keeps at least `NODE_MIN` of them.
const NODE = 32
const NODE_MIN = 8

// How many slots of a `KeyTree` node are in use, and a leaf's neighbours (-1 for none).
struct TreeNode:
    var size: -2_147_483_648..=2_147_483_647
    var next: -2_147_483_648..=2_147_483_647
    var prev: -2_147_483_648..=2_147_483_647

// One stored key: its value and copy count in a leaf, or a separator and its child's element
// count in an inner node.
struct Slot<K, V>:
    var key: K
    var val: V
    var cnt: Int

// The B+tree behind `SortedBag`, `SortedSet` and `SortedMap`. Node `u` owns slots
// `u * NODE .. u * NODE + len_of(u)` of `slots` (and of `kids`, which stays empty while the
// root is the only node). A leaf slot holds a key, its value and its copy count; an inner slot
// holds a separator key, its child's element count, and the child in `kids`.
// Separator `i >= 1` is at most every key of child `i` and above every key of child `i - 1`;
// separator 0 is never read. Leaves sit at depth `height` and link to their neighbours.
struct KeyTree<K, V>:
    var slots: List<Slot<K, V>>
    var kids: List<-2_147_483_648..=2_147_483_647>
    var nodes: List<TreeNode>
    var free: List<-2_147_483_648..=2_147_483_647>
    var root: Int
    var height: Int
    var head: Int
    var tail: Int
    var total: Int
    var live: Int

    fn new() -> KeyTree<K, V> = KeyTree([], [], [], [], -1, 0, -1, -1, 0, 0)

    // Move the tree out in O(1), leaving `self` empty.
    fn take(mut self) -> KeyTree<K, V>:
        let out = KeyTree(slots, kids, nodes, free, root, height, head, tail, total, live)
        slots = []
        kids = []
        nodes = []
        free = []
        root = -1
        height = 0
        head = -1
        tail = -1
        total = 0
        live = 0
        out

    // While the root leaf is the only node, it is node 0, it owns exactly `slots`, and `nodes`
    // stays empty; the second node fills `nodes` in.
    fn len_of(self, u: Int) -> Int:
        if nodes.len == 0: return slots.len
        unchecked: return nodes[u].size

    fn next_of(self, u: Int) -> Int:
        if nodes.len == 0: return -1
        unchecked: return nodes[u].next

    fn prev_of(self, u: Int) -> Int:
        if nodes.len == 0: return -1
        unchecked: return nodes[u].prev

    fn first_id(self) -> Int = if root < 0: -1 else: head * NODE

    fn last_id(self) -> Int:
        if root < 0: return -1
        unchecked: return tail * NODE + self.len_of(tail) - 1

    fn after(self, id: Int) -> Int:
        let u = id / NODE
        unchecked:
            if id % NODE + 1 < self.len_of(u): return id + 1
            let w = self.next_of(u)
            return if w < 0: -1 else: w * NODE

    fn before(self, id: Int) -> Int:
        if id % NODE > 0: return id - 1
        unchecked:
            let w = self.prev_of(id / NODE)
            return if w < 0: -1 else: w * NODE + self.len_of(w) - 1

    // How many of node `u`'s first `n` keys are `<= x` (`le`) or `< x`.
    fn count_keys(self, u: Int, n: Int, x: K, le: Bool) -> Int:
        var lo = 0
        var hi = n
        let base = u * NODE
        unchecked:
            if le:
                while lo < hi:
                    let m = (lo + hi) / 2
                    if x < slots[base + m].key: hi = m else: lo = m + 1
            else:
                while lo < hi:
                    let m = (lo + hi) / 2
                    if slots[base + m].key < x: lo = m + 1 else: hi = m
        lo

    // The child of inner node `u` whose keys may equal `x`.
    fn route(self, u: Int, x: K) -> Int:
        unchecked:
            let i = self.count_keys(u, self.len_of(u), x, true)
            return if i > 0: i - 1 else: 0

    // The slot of the first key `> x` (`le`) or `>= x`, or -1.
    fn seek(self, x: K, le: Bool) -> Int:
        if root < 0: return -1
        var u = root
        unchecked:
            for _ in 0..height: u = kids[u * NODE + self.route(u, x)]
            let j = self.count_keys(u, self.len_of(u), x, le)
            if j < self.len_of(u): return u * NODE + j
            let w = self.next_of(u)
            return if w < 0: -1 else: w * NODE

    // The slot of the last key `<= x` (`le`) or `< x`, or -1.
    fn seek_back(self, x: K, le: Bool) -> Int:
        let s = self.seek(x, le)
        if s < 0: self.last_id() else: self.before(s)

    // The slot holding `x`, or -1.
    fn find(self, x: K) -> Int:
        let s = self.seek(x, false)
        unchecked:
            if s >= 0 and not (x < slots[s].key): return s
        -1

    // How many elements are `<= x` (`le`) or `< x`.
    // Each node's element count `w` is known from its parent slot, so a level adds up the
    // shorter side of the split, and a leaf holding one copy per key adds nothing.
    fn rank_of(self, x: K, le: Bool) -> Int:
        if root < 0: return 0
        var ans = 0
        var u = root
        var w = total
        unchecked:
            for _ in 0..height:
                let i = self.route(u, x)
                ans += self.count_before(u, i, w)
                w = slots[u * NODE + i].cnt
                u = kids[u * NODE + i]
            let n = self.len_of(u)
            let j = self.count_keys(u, n, x, le)
            ans += if w == n: j else: self.count_before(u, j, w)
        ans

    // The elements in the first `i` slots of node `u`, which holds `w` in all.
    fn count_before(self, u: Int, i: Int, w: Int) -> Int:
        let base = u * NODE
        var s = 0
        unchecked:
            let n = self.len_of(u)
            if 2 * i <= n:
                for j in base..base + i: s += slots[j].cnt
                return s
            for j in base + i..base + n: s += slots[j].cnt
        w - s

    // The slot of the element at 0-based position `i` counting copies, or -1.
    fn nth_id(self, i: Int) -> Int:
        if i < 0 or i >= total: return -1
        var k = i
        var u = root
        unchecked:
            for _ in 0..height:
                var j = u * NODE
                while k >= slots[j].cnt:
                    k -= slots[j].cnt
                    j += 1
                u = kids[j]
            var j = u * NODE
            while k >= slots[j].cnt:
                k -= slots[j].cnt
                j += 1
            return j

    // The first slot of the subtree under inner slot `s`, one level above height `h`.
    fn leftmost(self, s: Int, h: Int) -> Int:
        var u = 0
        unchecked:
            u = kids[s]
            for _ in 0..h: u = kids[u * NODE]
        u * NODE

    // The first slot whose entry satisfies monotone `ok` (false then true), or -1. Calls `ok`
    // only on stored entries, a binary search of the children's first entries per level.
    fn first_true(self, ok: (K, V) -> Bool) -> Int:
        if root < 0: return -1
        var fallback = -1
        var u = root
        unchecked:
            for h in (0..height).rev:
                let base = u * NODE
                var lo = 0
                var hi = self.len_of(u)
                while lo < hi:
                    let m = (lo + hi) / 2
                    let f = self.leftmost(base + m, h)
                    if ok(slots[f].key, slots[f].val): hi = m else: lo = m + 1
                if lo == 0: return self.leftmost(base, h)
                if lo < self.len_of(u): fallback = self.leftmost(base + lo, h)
                u = kids[base + lo - 1]
            let base = u * NODE
            var lo = 0
            var hi = self.len_of(u)
            while lo < hi:
                let m = (lo + hi) / 2
                if ok(slots[base + m].key, slots[base + m].val): hi = m else: lo = m + 1
            return if lo < self.len_of(u): base + lo else: fallback

    // The last slot whose entry satisfies monotone `ok` (true then false), or -1.
    fn last_true(self, ok: (K, V) -> Bool) -> Int:
        if root < 0: return -1
        var u = root
        unchecked:
            for h in (0..height).rev:
                let base = u * NODE
                var lo = 0
                var hi = self.len_of(u)
                while lo < hi:
                    let m = (lo + hi) / 2
                    let f = self.leftmost(base + m, h)
                    if ok(slots[f].key, slots[f].val): lo = m + 1 else: hi = m
                if lo == 0: return -1
                u = kids[base + lo - 1]
            let base = u * NODE
            var lo = 0
            var hi = self.len_of(u)
            while lo < hi:
                let m = (lo + hi) / 2
                if ok(slots[base + m].key, slots[base + m].val): lo = m + 1 else: hi = m
            return if lo > 0: base + lo - 1 else: -1

    fn reserve(mut self, cap: Int):
        if cap <= 0: return
        if nodes.len == 0 and cap <= NODE:
            slots.reserve(cap)
            return
        let n = (cap + NODE_MIN) / (NODE / 2) + 1
        slots.reserve(n * NODE)
        kids.reserve(n * NODE)
        nodes.reserve(n)

    // A fresh node; `x` and `v` fill its slots until they are written. The lone root leaf grows
    // one slot at a time (`open`), with no `nodes` entry and an empty `kids`, so a small tree
    // holds only what it stores; all three are filled out when a second node is made.
    fn alloc(mut self, x: K, v: V) -> Int:
        unchecked:
            if let u = free.pop():
                nodes[u].size = 0
                nodes[u].next = -1
                nodes[u].prev = -1
                return u
            if nodes.len == 0:
                if root < 0: return 0
                nodes.push(TreeNode(slots.len, -1, -1))
            while slots.len < (nodes.len + 1) * NODE: self.grow(x, v)
            while kids.len < slots.len: kids.push(-1)
            nodes.push(TreeNode(0, -1, -1))
            return nodes.len - 1

    fn grow(mut self, x: K, v: V):
        slots.push(Slot(x, v, 0))

    // Swap slots `a` and `b` whole, so moving an entry never copies its key or value.
    fn trade(mut self, a: Int, b: Int):
        unchecked:
            slots.swap(a, b)
            if kids.len > 0: kids.swap(a, b)

    // Open slot `j` of node `u` by shifting the slots after it right.
    fn open(mut self, u: Int, j: Int, x: K, v: V):
        let base = u * NODE
        let n = self.len_of(u)
        if nodes.len == 0: self.grow(x, v)
        unchecked:
            for t in (j..n).rev: self.trade(base + t, base + t + 1)
            if nodes.len > 0: nodes[u].size += 1

    // Close slot `j` of node `u` by shifting the slots after it left.
    fn close(mut self, u: Int, j: Int):
        let base = u * NODE
        unchecked:
            for t in j + 1..self.len_of(u): self.trade(base + t - 1, base + t)
            if nodes.len == 0:
                let _ = slots.pop()
            else:
                nodes[u].size -= 1

    fn weight(self, u: Int) -> Int:
        var s = 0
        unchecked:
            for j in u * NODE..u * NODE + self.len_of(u): s += slots[j].cnt
        s

    // Move the upper half of full node `u` into a new right sibling and return it.
    fn split_node(mut self, u: Int, leaf: Bool, x: K, v: V) -> Int:
        let w = self.alloc(x, v)
        let half = NODE / 2
        unchecked:
            for t in 0..NODE - half: self.trade(w * NODE + t, u * NODE + half + t)
            nodes[w].size = NODE - half
            nodes[u].size = half
            if leaf:
                nodes[w].next = self.next_of(u)
                nodes[w].prev = u
                if self.next_of(u) >= 0: nodes[self.next_of(u)].prev = w else: tail = w
                nodes[u].next = w
        w

    // Add `c` copies of `x` under node `u` at height `h`; `mode` 0 adds copies, 1 keeps an
    // existing key unchanged, 2 overwrites its value. Returns the elements added and the new
    // right sibling of `u`, or -1.
    fn insert_at(mut self, u: Int, h: Int, x: K, v: V, c: Int, mode: Int) -> (Int, Int):
        unchecked:
            if h == 0:
                let j = self.count_keys(u, self.len_of(u), x, false)
                let s = u * NODE + j
                if j < self.len_of(u) and not (x < slots[s].key):
                    if mode == 1: return (0, -1)
                    if mode == 2:
                        slots[s].val = v
                        return (0, -1)
                    slots[s].cnt += c
                    return (c, -1)
                live += 1
                if self.len_of(u) < NODE:
                    self.put(u, j, x, v, c, -1)
                    return (c, -1)
                let w = self.split_node(u, true, x, v)
                if j <= self.len_of(u): self.put(u, j, x, v, c, -1) else: self.put(w, j - self.len_of(u), x, v, c, -1)
                return (c, w)
            let i = self.route(u, x)
            let s = u * NODE + i
            let (d, w) = self.insert_at(kids[s], h - 1, x, v, c, mode)
            slots[s].cnt += d
            if w < 0: return (d, -1)
            let wc = self.weight(w)
            slots[s].cnt -= wc
            if self.len_of(u) < NODE:
                self.put(u, i + 1, slots[w * NODE].key, v, wc, w)
                return (d, -1)
            let z = self.split_node(u, false, x, v)
            if i + 1 <= self.len_of(u): self.put(u, i + 1, slots[w * NODE].key, v, wc, w)
            else: self.put(z, i + 1 - self.len_of(u), slots[w * NODE].key, v, wc, w)
            return (d, z)

    fn put(mut self, u: Int, j: Int, x: K, v: V, c: Int, kid: Int):
        self.open(u, j, x, v)
        let s = u * NODE + j
        unchecked:
            slots[s].key = x
            slots[s].val = v
            slots[s].cnt = c
            if kids.len > 0: kids[s] = kid

    // Add `c` copies of `x` with value `v` (see `insert_at` for `mode`); returns the elements added.
    fn insert(mut self, x: K, v: V, c: Int, mode: Int) -> Int:
        unchecked:
            if root < 0:
                root = self.alloc(x, v)
                head = root
                tail = root
                height = 0
                self.put(root, 0, x, v, c, -1)
                live += 1
                total += c
                return c
            let (d, w) = self.insert_at(root, height, x, v, c, mode)
            total += d
            if w >= 0:
                let r = self.alloc(x, v)
                self.put(r, 0, slots[root * NODE].key, v, total - self.weight(w), root)
                self.put(r, 1, slots[w * NODE].key, v, self.weight(w), w)
                root = r
                height += 1
            return d

    // Remove one copy of `x` (all copies when `all`) under node `u` at height `h`; returns
    // how many were removed.
    fn erase_at(mut self, u: Int, h: Int, x: K, all: Bool) -> Int:
        unchecked:
            if h == 0:
                let j = self.count_keys(u, self.len_of(u), x, false)
                let s = u * NODE + j
                if j == self.len_of(u) or x < slots[s].key: return 0
                if not all and slots[s].cnt > 1:
                    slots[s].cnt -= 1
                    return 1
                let r = slots[s].cnt
                self.close(u, j)
                live -= 1
                return r
            let i = self.route(u, x)
            let s = u * NODE + i
            let r = self.erase_at(kids[s], h - 1, x, all)
            if r == 0: return 0
            slots[s].cnt -= r
            if self.len_of(kids[s]) < NODE_MIN: self.refill(u, i, h - 1)
            return r

    // Child `i` of inner node `u` (at height `h`) is short: merge it with a sibling, or even
    // the two out when together they overfill one node.
    fn refill(mut self, u: Int, i: Int, h: Int):
        let l = if i + 1 < self.len_of(u): i else: i - 1
        let ls = u * NODE + l
        let rs = ls + 1
        unchecked:
            let a = kids[ls]
            let b = kids[rs]
            let na = self.len_of(a)
            let nb = self.len_of(b)
            // Separator 0 of `b` is about to sit past slot 0: give it `b`'s real lower bound.
            if h > 0: slots[b * NODE].key = slots[rs].key
            if na + nb <= NODE:
                for t in 0..nb: self.trade(a * NODE + na + t, b * NODE + t)
                nodes[a].size = na + nb
                nodes[b].size = 0
                slots[ls].cnt += slots[rs].cnt
                if h == 0:
                    nodes[a].next = self.next_of(b)
                    if self.next_of(b) >= 0: nodes[self.next_of(b)].prev = a else: tail = a
                self.close(u, l + 1)
                free.push(b)
                return
            var moved = 0
            if na < nb:
                let k = (nb - na) / 2
                for t in 0..k:
                    moved += slots[b * NODE + t].cnt
                    self.trade(a * NODE + na + t, b * NODE + t)
                for t in k..nb: self.trade(b * NODE + t - k, b * NODE + t)
                nodes[a].size = na + k
                nodes[b].size = nb - k
                slots[ls].cnt += moved
                slots[rs].cnt -= moved
            else:
                let k = (na - nb) / 2
                for t in (0..nb).rev: self.trade(b * NODE + t + k, b * NODE + t)
                for t in 0..k:
                    moved += slots[a * NODE + na - k + t].cnt
                    self.trade(b * NODE + t, a * NODE + na - k + t)
                nodes[a].size = na - k
                nodes[b].size = nb + k
                slots[ls].cnt -= moved
                slots[rs].cnt += moved
            slots[rs].key = slots[b * NODE].key

    // Remove one copy of `x` (every copy when `all`); returns how many were removed.
    fn erase(mut self, x: K, all: Bool) -> Int:
        if root < 0: return 0
        let r = self.erase_at(root, height, x, all)
        total -= r
        unchecked:
            if height > 0 and self.len_of(root) == 1:
                free.push(root)
                root = kids[root * NODE]
                height -= 1
            elif height == 0 and self.len_of(root) == 0:
                if nodes.len > 0: free.push(root)
                root = -1
                head = -1
                tail = -1
        r

/// An ordered multiset (bag) of `K` backed by a B+tree of 32-slot nodes in flat arrays, with
/// recycled node slots and linked leaves for \(O(1)\) cursor stepping.
/// Keeps one leaf slot per distinct key with its multiplicity; inner slots keep subtree element counts.
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
    var t: KeyTree<K, ()>

    /// Create an empty `SortedBag<K>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag(); b.len == 0`
    pub fn new() -> SortedBag<K> = SortedBag(KeyTree<K, ()>.new())

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
        b.reserve(xs.len)
        for x in xs: b.add(x)
        b

    /// Total number of elements in the bag (counting duplicates).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).len == 3`
    @get pub fn len(self) -> Int:
        t.total

    /// Number of distinct keys currently in the bag.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).distinct == 2`
    @get pub fn distinct(self) -> Int:
        t.live

    /// Whether the bag has no elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SortedBag.of([1]).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        t.root < 0

    /// Remove all elements from the bag.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); b.clear(); b.len == 0 and b.first == None`
    pub fn clear(mut self):
        let _ = t.take()

    /// Move the bag's contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var b = SortedBag.of([1, 2]); let t = b.take(); b.is_empty and t.to_list() == [1, 2]`
    pub fn take(mut self) -> SortedBag<K>:
        SortedBag(t.take())

    /// Insert one copy of `x` into the bag. Always returns `true`.
    ///
    /// The `Bool` matches `SortedSet.add`, so code can switch between the two.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b: SortedBag<Int> = SortedBag(); b.add(3) and b.add(3) and b.count(3) == 2`
    pub fn add(mut self, x: K) -> Bool:
        let _ = t.insert(x, (), 1, 0)
        true

    fn add_unique(mut self, x: K) -> Bool = t.insert(x, (), 1, 1) > 0

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
        let _ = t.insert(x, (), c, 0)

    /// Remove one copy of `x` if present; returns `true` if a copy was removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2]); b.remove(2) and b.count(2) == 1 and not b.remove(7)`
    pub fn remove(mut self, x: K) -> Bool = t.erase(x, false) > 0

    /// Remove all copies of `x` and return how many were removed.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var b = SortedBag.of([2, 2, 3]); b.remove_all(2) == 2 and b.to_list() == [3]`
    /// example: `var b = SortedBag.of([3]); b.remove_all(2) == 0`
    pub fn remove_all(mut self, x: K) -> Int = t.erase(x, true)

    /// Remove one copy of the key at cursor `c`, as `remove(key_of(c))` would.
    ///
    /// `c` must come from this bag and no mutation may have happened since it was made.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never in release (a stale cursor removes an unspecified key); debug traps when the node of `c` has been freed
    /// example: `var b = SortedBag.of([2, 5, 5]); b.remove_at(b.cursor_ge(3)!); b.to_list() == [2, 5]`
    pub fn remove_at(mut self, c: BagCursor):
        self.remove(t.slots[c.id].key)

    /// How many copies of `x` are currently in the bag.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `SortedBag.of([2, 2, 3]).count(2) == 2`
    /// example: `SortedBag.of([2, 2, 3]).count(4) == 0`
    pub fn count(self, x: K) -> Int:
        let s = t.find(x)
        if s < 0: return 0
        unchecked: return t.slots[s].cnt

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
    @get pub fn first(self) -> ?K:
        if t.root < 0: return None
        unchecked: return t.slots[t.first_id()].key

    /// The largest element, or `None` when empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `SortedBag<Int>().last == None`
    /// ties: `SortedBag.of([3, 1, 3]).last == Some(3)`
    /// traps: never
    /// example: `SortedBag.of([3, 1, 3]).last == Some(3)`
    @get pub fn last(self) -> ?K:
        if t.root < 0: return None
        unchecked: return t.slots[t.last_id()].key

    /// Remove and return one copy of the smallest element, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b = SortedBag<Int>(); b.pop_first() == None`
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
    /// empty: `var b = SortedBag<Int>(); b.take_first(x => true) == None`
    /// ties: `var b = SortedBag.of([4, 4]); b.take_first(x => x >= 3) == Some(4) and b.count(4) == 1`
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
    /// empty: `var b = SortedBag.of([1, 2]); var e = SortedBag<Int>(); b.drain_range(2..2).collect() == [] and e.drain_range(0..9).collect() == [] and b.len == 2`
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
    /// empty: `var b = SortedBag<Int>(); b.pop_last() == None`
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
    /// empty: `SortedBag<Int>().rank(5) == 0`
    /// traps: never
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(2) == 1`
    /// example: `SortedBag.of([1, 2, 2, 5]).rank(3) == 3`
    pub fn rank(self, k: K) -> Int = t.rank_of(k, false)

    fn rank_le(self, k: K) -> Int = t.rank_of(k, true)

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
        SortedRange(self, Some(r.l), true, Some(r.r), false)

    fn range_first(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K:
        let s = if let a = lo: t.seek(a, not lo_inc) else: t.first_id()
        if s < 0: return None
        unchecked:
            let k = t.slots[s].key
            return if self.hi_ok(k, hi, hi_inc): k else: None

    fn range_last(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> ?K:
        let s = if let b = hi: t.seek_back(b, hi_inc) else: t.last_id()
        if s < 0: return None
        unchecked:
            let k = t.slots[s].key
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
        var s = if let a = lo: t.seek(a, not lo_inc) else: t.first_id()
        unchecked:
            while s >= 0:
                let k = t.slots[s].key
                if not self.hi_ok(k, hi, hi_inc): break
                for _ in 0..t.slots[s].cnt: yield k
                s = t.after(s)

    fn range_each_rev(self, lo: ?K, lo_inc: Bool, hi: ?K, hi_inc: Bool) -> yields K:
        var s = if let b = hi: t.seek_back(b, hi_inc) else: t.last_id()
        unchecked:
            while s >= 0:
                let k = t.slots[s].key
                if not self.lo_ok(k, lo, lo_inc): break
                for _ in 0..t.slots[s].cnt: yield k
                s = t.before(s)

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
    pub fn nth(self, i: Int) -> ?K:
        let s = t.nth_id(i)
        if s < 0: return None
        unchecked: return t.slots[s].key

    /// Smallest key where monotone `ok(k)` holds (`false..false, true..true`), or `None`.
    /// Checks monotonicity at the boundary. \(O(\log n)\) expected.
    ///
    /// `ok` is called on keys along one root-to-leaf path. Debug builds check that `ok` is
    /// false on the key just below the answer.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `SortedBag<Int>().first_true(x => true) == None`
    /// ties: `SortedBag.of([5, 4, 4]).first_true(x => x >= 3) == Some(4)`
    /// traps: never; an `ok` that is not false-then-true gives an unspecified answer
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).first_true(x => x > 9) == None`
    pub fn first_true(self, ok: K -> Bool) -> ?K:
        let s = t.first_true((k, _) => ok(k))
        if s < 0: return None
        unchecked: return t.slots[s].key

    /// Largest key where monotone `ok(k)` holds (`true..true, false..false`), or `None`.
    /// Checks monotonicity at the boundary. \(O(\log n)\) expected.
    ///
    /// cost: \(O(\log n)\) expected calls of `ok`
    /// empty: `SortedBag<Int>().last_true(x => true) == None`
    /// ties: `SortedBag.of([1, 4, 4]).last_true(x => x <= 5) == Some(4)`
    /// traps: never; an `ok` that is not true-then-false gives an unspecified answer
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)`
    /// example: `SortedBag.of([1, 4, 6]).last_true(x => x < 1) == None`
    pub fn last_true(self, ok: K -> Bool) -> ?K:
        let s = t.last_true((k, _) => ok(k))
        if s < 0: return None
        unchecked: return t.slots[s].key

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
    /// empty: `SortedBag<Int>().cursor_first() == None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_first()!) == 2`
    pub fn cursor_first(self) -> ?BagCursor = self.cursor(t.first_id())

    /// Cursor to the largest key, or `None` when empty.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_last() == None`
    /// traps: never
    /// example: `let b = SortedBag.of([5, 2, 2]); b.key_of(b.cursor_last()!) == 5`
    pub fn cursor_last(self) -> ?BagCursor = self.cursor(t.last_id())

    /// Cursor to exact key `k`, or `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_at(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.cursor_at(5) != None and b.cursor_at(3) == None`
    pub fn cursor_at(self, k: K) -> ?BagCursor = self.cursor(t.find(k))

    /// Cursor to smallest key `>= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_ge(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_ge(3)!) == 5 and b.cursor_ge(6) == None`
    pub fn cursor_ge(self, k: K) -> ?BagCursor = self.cursor(t.seek(k, false))

    /// Cursor to smallest key `> k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_gt(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_gt(2)!) == 5 and b.cursor_gt(5) == None`
    pub fn cursor_gt(self, k: K) -> ?BagCursor = self.cursor(t.seek(k, true))

    /// Cursor to largest key `<= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_le(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_le(4)!) == 2 and b.cursor_le(1) == None`
    pub fn cursor_le(self, k: K) -> ?BagCursor = self.cursor(t.seek_back(k, true))

    /// Cursor to largest key `< k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `SortedBag<Int>().cursor_lt(1) == None`
    /// traps: never
    /// example: `let b = SortedBag.of([2, 5]); b.key_of(b.cursor_lt(5)!) == 2 and b.cursor_lt(2) == None`
    pub fn cursor_lt(self, k: K) -> ?BagCursor = self.cursor(t.seek_back(k, false))

    /// Key stored at cursor `c`.
    ///
    /// `c` must come from this bag and no mutation may have happened since it was made.
    ///
    /// cost: \(O(1)\)
    /// traps: never in release (a stale cursor reads an unspecified key); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.cursor_first()!) == 2`
    pub fn key_of(self, c: BagCursor) -> K = t.slots[c.id].key

    /// Multiplicity stored at cursor `c`.
    ///
    /// cost: \(O(1)\)
    /// traps: never in release (a stale cursor reads an unspecified count); debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.count_of(b.cursor_at(2)!) == 2`
    pub fn count_of(self, c: BagCursor) -> Int = t.slots[c.id].cnt

    /// Cursor to the next larger distinct key, or `None`.
    ///
    /// Steps over all copies of the current key at once.
    ///
    /// cost: \(O(1)\) amortized, \(O(\log n)\) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 2, 5]); b.key_of(b.next_cursor(b.cursor_first()!)!) == 5`
    /// example: `let b = SortedBag.of([2, 5]); b.next_cursor(b.cursor_last()!) == None`
    pub fn next_cursor(self, c: BagCursor) -> ?BagCursor = self.cursor(t.after(c.id))

    /// Cursor to the next smaller distinct key, or `None`.
    ///
    /// cost: \(O(1)\) amortized, \(O(\log n)\) worst case
    /// traps: never in release; debug traps when the node of `c` has been freed
    /// example: `let b = SortedBag.of([2, 5, 5]); b.key_of(b.prev_cursor(b.cursor_last()!)!) == 2`
    /// example: `let b = SortedBag.of([2, 5]); b.prev_cursor(b.cursor_first()!) == None`
    pub fn prev_cursor(self, c: BagCursor) -> ?BagCursor = self.cursor(t.before(c.id))

    /// Visit every distinct `(key, count)` in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().entries().collect() == []`
    /// order: ascending key, one entry per distinct key
    /// traps: never
    /// example: `[e.key for e in SortedBag.of([5, 2, 2]).entries()] == [2, 5]`
    /// example: `[e.count for e in SortedBag.of([5, 2, 2]).entries()] == [2, 1]`
    pub fn entries(self) -> yields (key: K, count: Int):
        var s = t.first_id()
        unchecked:
            while s >= 0:
                yield (key: t.slots[s].key, count: t.slots[s].cnt)
                s = t.after(s)

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
        var s = t.last_id()
        unchecked:
            while s >= 0:
                yield (key: t.slots[s].key, count: t.slots[s].cnt)
                s = t.before(s)

    /// Visit every element, duplicates included, in ascending order; `for x in b` uses this.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().each().collect() == []`
    /// order: ascending, each key `count` times
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2]).each().collect() == [2, 2, 5]`
    pub fn each(self) -> yields K:
        for k in self.range_each(None, false, None, false): yield k

    /// Visit every element, duplicates included, in descending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `SortedBag<Int>().each_rev().collect() == []`
    /// order: descending, each key `count` times
    /// traps: never
    /// example: `SortedBag.of([5, 2, 2]).each_rev().collect() == [5, 2, 2]`
    pub fn each_rev(self) -> yields K:
        for k in self.range_each_rev(None, false, None, false): yield k

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

    /// Preallocate node storage for at least `cap` distinct keys.
    ///
    /// cost: \(O(n)\) when the pool grows (it moves the nodes), \(O(1)\) otherwise
    /// traps: never
    /// example: `var b = SortedBag.of([1]); b.reserve(100); b.to_list() == [1]`
    pub fn reserve(mut self, cap: Int) = t.reserve(cap)

    /// Split off and return all elements `>= k`, keeping `< k` in `self`.
    ///
    /// Every copy of a key goes to the same side.
    ///
    /// The larger side keeps its storage; the smaller side's keys move to a fresh one.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's distinct keys
    /// empty: `var b = SortedBag<Int>(); let hi = b.split_at(3); b.len == 0 and hi.len == 0`
    /// traps: never
    /// example: `var b = SortedBag.of([1, 3, 3, 5]); let hi = b.split_at(3); b.to_list() == [1] and hi.to_list() == [3, 3, 5]`
    pub fn split_at(mut self, k: K) -> SortedBag<K>:
        var out = SortedBag<K>.new()
        if 2 * t.rank_of(k, false) < t.total:
            out = self.take()
            while let x = out.first:
                if not (x < k): break
                let _ = t.insert(x, (), out.t.erase(x, true), 0)
        else:
            while let x = self.last:
                if x < k: break
                let _ = out.t.insert(x, (), t.erase(x, true), 0)
        out

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The inverse of `split_at`. The precondition is checked in \(O(1)\) against `self.last` and
    /// `other.first`; either side may be empty.
    ///
    /// The smaller side's keys move into the larger side's storage, as in `split_at`.
    ///
    /// cost: \(O(\log n + s \log n)\) expected, `s` the smaller side's distinct keys
    /// traps: when both are non-empty and `self.last >= other.first`
    /// example: `var a = SortedBag.of([1, 2]); a.join(SortedBag.of([4, 4])); a.to_list() == [1, 2, 4, 4]`
    pub fn join(mut self, other: SortedBag<K>):
        if let a = self.last:
            if let b = other.first: assert a < b, "`join` needs every key of `self` below every key of `other`"
        var o = other
        if t.total < o.t.total:
            let mine = t.take()
            t = o.t.take()
            o.t = mine
        for (k, c) in o.entries(): let _ = t.insert(k, (), c, 0)

    /// Merge all elements of `other` into `self` (small-to-large, emptying `other`).
    ///
    /// Copies add up: a key with count `a` in `self` and `b` in `other` ends with `a + b`. The
    /// smaller bag's elements are inserted into the larger, whichever side that is.
    ///
    /// cost: \(O(\min(n, m) \log(n + m))\) expected
    /// empty: `var a = SortedBag.of([1, 2]); var e = SortedBag<Int>(); a.absorb(mut e); a.to_list() == [1, 2]`
    /// traps: never
    /// example: `var a = SortedBag.of([1, 3]); var b = SortedBag.of([3, 2]); a.absorb(mut b); a.to_list() == [1, 2, 3, 3] and b.is_empty`
    pub fn absorb(mut self, mut other: SortedBag<K>):
        if t.total < other.t.total:
            let mine = t.take()
            t = other.t.take()
            other.t = mine
        for (k, c) in other.entries(): let _ = t.insert(k, (), c, 0)
        other.clear()

    /// Sum of all elements (for additive `K`). \(O(1)\) when augmented, \(O(n)\) otherwise.
    ///
    /// Each copy is added. Nodes keep no subtree sum, so this bag is never augmented and
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

    fn cursor(self, s: Int) -> ?BagCursor = if s >= 0: BagCursor(s) else: None

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
        s.reserve(xs.len)
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
    pub fn pop_first(mut self) -> ?K = bag.pop_first()

    /// Remove and return the largest key, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var s = SortedSet<Int>(); s.pop_last() == None`
    /// traps: never
    /// example: `var s = SortedSet.of([4, 1]); s.pop_last() == Some(4) and s.to_list() == [1]`
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

    /// Preallocate node storage for at least `cap` keys.
    ///
    /// cost: \(O(n)\) when the pool grows (it moves the nodes), \(O(1)\) otherwise
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

/// Order-statistic ordered map from `K` to `V`, keys and values side by side in the `SortedBag` B+tree.
/// `get`, `set`, `remove`, `rank`, `nth`, `first_ge`/`last_le`, `first_true`/`last_true`, `split_at`, `join` are \(O(\log n)\).
///
/// Entries are ordered by key; keys are distinct, so no lookup has a tie to break. Every
/// entry-returning method gives `(key, val)`.
pub struct SortedMap<K, V>:
    var t: KeyTree<K, V>

    fn entry(self, s: Int) -> ?(K, V):
        if s < 0: return None
        unchecked: return (t.slots[s].key, t.slots[s].val)

    /// Create an empty `SortedMap<K, V>`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.len == 0`
    pub fn new() -> SortedMap<K, V>:
        SortedMap(KeyTree<K, V>.new())

    /// Reserve node storage for at least `cap` entries.
    ///
    /// cost: \(O(\mathit{cap})\) when the storage grows, \(O(1)\) otherwise
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.reserve(100); m.get(1) == Some("a")`
    pub fn reserve(mut self, cap: Int):
        t.reserve(cap)

    /// Number of key-value pairs.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(1, "b"); m.len == 1`
    @get pub fn len(self) -> Int:
        t.total

    /// Whether the map is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.is_empty`
    @get pub fn is_empty(self) -> Bool:
        t.root < 0

    /// Remove all entries.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.clear(); m.get(1) == None`
    pub fn clear(mut self):
        let _ = t.take()

    /// Move contents out in \(O(1)\), leaving `self` empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); let t = m.take(); m.is_empty and t.get(1) == Some("a")`
    pub fn take(mut self) -> SortedMap<K, V>:
        SortedMap(t.take())

    /// Look up `k`, returning `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.get(1) == Some("a") and m.get(2) == None`
    pub fn get(self, k: K) -> ?V:
        let s = t.find(k)
        if s < 0: return None
        unchecked: return t.slots[s].val

    /// Whether `k` is present.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.contains(1) and not m.contains(2)`
    pub fn contains(self, k: K) -> Bool:
        t.find(k) >= 0

    /// Insert or overwrite `k -> v` (W8 `IndexSet` backing `m[k] = v`).
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m[1] = "a"; m[1] = "b"; m.get(1) == Some("b")`
    pub fn set(mut self, k: K, v: V):
        let _ = t.insert(k, v, 1, 2)

    /// Remove `k` and return its previous value, or `None` if absent.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.remove(1) == Some("a") and m.remove(1) == None`
    pub fn remove(mut self, k: K) -> ?V:
        let v = self.get(k)
        if v != None: let _ = t.erase(k, true)
        v

    /// Smallest `(key, val)` pair, or `None` if empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(1, "b"); m.first == Some((1, "b")) and m.len == 1`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.first == Some((1, "a"))`
    @get pub fn first(self) -> ?(K, V):
        self.entry(t.first_id())

    /// Largest `(key, val)` pair, or `None` if empty. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(2, "a"); m.set(2, "b"); m.last == Some((2, "b")) and m.len == 1`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.last == Some((3, "c"))`
    @get pub fn last(self) -> ?(K, V):
        self.entry(t.last_id())

    /// Remove and return the smallest `(key, val)` pair, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.pop_first() == None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.pop_first() == Some((1, "a")) and m.len == 1`
    pub fn pop_first(mut self) -> ?(K, V):
        let e = self.first
        if let (k, _) = e: let _ = t.erase(k, true)
        e

    /// Remove and return the largest `(key, val)` pair, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.pop_last() == None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.pop_last() == Some((3, "c")) and m.len == 1`
    pub fn pop_last(mut self) -> ?(K, V):
        let e = self.last
        if let (k, _) = e: let _ = t.erase(k, true)
        e

    /// Smallest entry with key `>= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first_ge(0) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.first_ge(2) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.first_ge(2) == Some((3, "c")) and m.first_ge(3) == Some((3, "c"))`
    pub fn first_ge(self, k: K) -> ?(K, V):
        self.entry(t.seek(k, false))

    /// Smallest entry with key `> k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.first_gt(0) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.first_gt(2) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.first_gt(1) == Some((3, "c")) and m.first_gt(3) == None`
    pub fn first_gt(self, k: K) -> ?(K, V):
        self.entry(t.seek(k, true))

    /// Largest entry with key `<= k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last_le(9) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.last_le(4) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.last_le(2) == Some((1, "a")) and m.last_le(0) == None`
    pub fn last_le(self, k: K) -> ?(K, V):
        self.entry(t.seek_back(k, true))

    /// Largest entry with key `< k`, or `None`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.last_lt(9) == None`
    /// ties: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "a"); m.set(3, "b"); m.last_lt(4) == Some((3, "b"))`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.last_lt(3) == Some((1, "a")) and m.last_lt(1) == None`
    pub fn last_lt(self, k: K) -> ?(K, V):
        self.entry(t.seek_back(k, false))

    /// Number of entries with key strictly less than `k`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.rank(1) == 0`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(1, "a"); m.set(3, "c"); m.rank(3) == 1 and m.rank(9) == 2`
    pub fn rank(self, k: K) -> Int:
        t.rank_of(k, false)

    /// The `i`-th entry in key order (0-based), or `None`.
    ///
    /// `None` for any `i` outside `0..len`, negative included.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.nth(0) == None`
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.nth(1) == Some((3, "c")) and m.nth(2) == None`
    pub fn nth(self, i: Int) -> ?(K, V):
        self.entry(t.nth_id(i))

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
        self.entry(t.first_true(ok))

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
        self.entry(t.last_true(ok))

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
            let s = if let b = before: t.seek_back(b, false)
                elif let a = after: t.seek(a, true)
                else: t.first_id()
            let (k, v) = self.entry(s) ?? break
            if not ok(k, v): break
            let _ = t.erase(k, true)
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
        if 2 * t.rank_of(k, false) < t.total:
            out = self.take()
            while let (x, v) = out.first:
                if not (x < k): break
                let _ = out.t.erase(x, true)
                let _ = t.insert(x, v, 1, 0)
        else:
            while let (x, v) = self.last:
                if x < k: break
                let _ = t.erase(x, true)
                let _ = out.t.insert(x, v, 1, 0)
        out

    /// Concatenate `other` onto `self` when every key in `self` is `<` every key in `other`.
    ///
    /// The precondition is checked in \(O(1)\) against the last key of `self` and the first of
    /// `other`.
    ///
    /// cost: as `SortedBag.join` for the keys, plus \(O(m)\) expected to move the `m` values of `other`
    /// traps: when both are non-empty and the last key of `self` is `>=` the first key of `other`
    /// example: `var a: SortedMap<Int, Str> = SortedMap(); a.set(1, "a"); var b: SortedMap<Int, Str> = SortedMap(); b.set(3, "c"); a.join(b); a.len == 2`
    pub fn join(mut self, other: SortedMap<K, V>):
        if let (a, _) = self.last:
            if let (b, _) = other.first: assert a < b, "`join` needs every key of `self` below every key of `other`"
        var o = other
        if t.total < o.t.total:
            let mine = t.take()
            t = o.t.take()
            o.t = mine
        for (k, v) in o.items: let _ = t.insert(k, v, 1, 0)

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
            let big = small.t.take()
            small.t = t.take()
            t = big
            flip = true
        for (k, v) in small.items:
            let s = t.find(k)
            if s < 0:
                let _ = t.insert(k, v, 1, 0)
                continue
            unchecked:
                if let j = join: t.slots[s].val = if flip: j(v, t.slots[s].val) else: j(t.slots[s].val, v)
                elif not flip: t.slots[s].val = v

    /// \(O(1)\) to make. Yields `(key, val)` pairs in ascending key order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.items.collect() == []`
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.items.collect() == [(1, "a"), (3, "c")]`
    @get pub fn items(self) -> yields (K, V):
        var s = t.first_id()
        unchecked:
            while s >= 0:
                yield (t.slots[s].key, t.slots[s].val)
                s = t.after(s)

    /// Yield `(key, val)` pairs in descending key order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.each_rev().collect() == []`
    /// order: descending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.each_rev().collect() == [(3, "c"), (1, "a")]`
    pub fn each_rev(self) -> yields (K, V):
        var s = t.last_id()
        unchecked:
            while s >= 0:
                yield (t.slots[s].key, t.slots[s].val)
                s = t.before(s)

    /// The map's keys, in ascending order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.keys.collect() == []`
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.keys.collect() == [1, 3]`
    @get pub fn keys(self) -> yields K:
        var s = t.first_id()
        unchecked:
            while s >= 0:
                yield t.slots[s].key
                s = t.after(s)

    /// The map's values, in ascending order of their keys.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) amortized per step
    /// empty: `var m: SortedMap<Int, Str> = SortedMap(); m.values.collect() == []`
    /// order: ascending key
    /// traps: never
    /// example: `var m: SortedMap<Int, Str> = SortedMap(); m.set(3, "c"); m.set(1, "a"); m.values.collect() == ["a", "c"]`
    @get pub fn values(self) -> yields V:
        var s = t.first_id()
        unchecked:
            while s >= 0:
                yield t.slots[s].val
                s = t.after(s)

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
        var t = start
        while t >= 0 and not (keys[t] > lo or (not strict and keys[t] == lo)): t = rs[t]
        if t < 0: return id
        var res = op(leaf(keys[t]), self.sub(rs[t]))
        t = ls[t]
        while t >= 0:
            if keys[t] > lo or (not strict and keys[t] == lo):
                res = op(op(leaf(keys[t]), self.sub(rs[t])), res)
                t = ls[t]
            else: t = rs[t]
        res

    fn fold_upto(self, start: Int, hi: K, strict: Bool) -> T:
        var t = start
        while t >= 0 and not (keys[t] < hi or (not strict and keys[t] == hi)): t = ls[t]
        if t < 0: return id
        var res = op(self.sub(ls[t]), leaf(keys[t]))
        t = rs[t]
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
    /// cost: \(O(n)\) when the pool grows (it moves the nodes), \(O(1)\) otherwise
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.reserve(100); b.len == 0`
    pub fn reserve(mut self, cap: Int):
        assert cap >= 0
        keys.reserve(cap)
        pri.reserve(cap)
        ls.reserve(cap)
        rs.reserve(cap)
        sz.reserve(cap)
        agg.reserve(cap)

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
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.all == 0`
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
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.rank(3) == 0`
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
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.nth(0) == None`
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
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.first == None`
    /// ties: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(1); b.first == Some(1)`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.first == Some(1)`
    @get pub fn first(self) -> ?K = lo_k

    /// Largest element, or `None`. \(O(1)\), kept current by every update.
    ///
    /// cost: \(O(1)\)
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.last == None`
    /// ties: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.last == Some(3)`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.last == Some(3)`
    @get pub fn last(self) -> ?K = hi_k

    /// Fold elements with key `< hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_lt(5) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_lt(3) == 1`
    pub fn fold_lt(self, hi: K) -> T = self.fold_upto(root, hi, true)

    /// Fold elements with key `<= hi`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_le(5) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_le(3) == 7`
    pub fn fold_le(self, hi: K) -> T = self.fold_upto(root, hi, false)

    /// Fold elements with key `>= lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_ge(0) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_ge(2) == 6`
    pub fn fold_ge(self, lo: K) -> T = self.fold_from(root, lo, false)

    /// Fold elements with key `> lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.fold_gt(0) == 0`
    /// traps: never
    /// example: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.add(3); b.add(1); b.add(3); b.fold_gt(1) == 6 and b.fold_gt(3) == 0`
    pub fn fold_gt(self, lo: K) -> T = self.fold_from(root, lo, true)

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
        if not (lo < hi): return id
        var t = root
        while t >= 0 and not (lo <= keys[t] and keys[t] < hi):
            t = if keys[t] < lo: rs[t] else: ls[t]
        if t < 0: return id
        op(op(self.fold_from(ls[t], lo, false), leaf(keys[t])), self.fold_upto(rs[t], hi, true))

    /// Yield all elements in ascending order.
    ///
    /// cost: \(O(1)\) amortized per step
    /// empty: `var b: AugSortedBag<Int, Int> = AugSortedBag((x, y) => x + y, 0, k => k); b.each().collect() == []`
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
    /// cost: \(O(n)\) when the pool grows (it moves the nodes), \(O(1)\) otherwise
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
    /// empty: `var b = BucketCount(0..0); b.add(0); b.len == 0` (traps R0401)
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
    /// empty: `BucketCount(0..10).count_lt(5) == 0`
    /// traps: never
    /// example: `var b = BucketCount(0..10); b.add(3); b.add(7, 2); b.count_lt(7) == 1 and b.count_lt(8) == 3`
    pub fn count_lt(self, x: Int) -> Int:
        if x <= lo: return 0
        if x >= hi: return total
        let i = x - lo
        sum(blk[b] for b in 0..i / bs) + sum(cnt[j] for j in i / bs * bs..i)

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
        var j = b * bs
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
            for i in k * bs..min((k + 1) * bs, cnt.len):
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

// A bag over 0..k against a count table, through enough keys for three B+tree levels: node
// splits while it grows, borrows and merges while it shrinks to empty, and every query near
// the 16- and 32-slot node boundaries.
fn check_bag_against_counts(b: SortedBag<Int>, cnt: List<Int>):
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
    for i in [0, 1, 15, 16, 17, 31, 32, 33, 47, 48, 63, 64, 65, plain.len / 2, plain.len - 1, plain.len]:
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
        if x < cnt.len:
            assert b.count(x) == cnt[x]
            below += cnt[x]

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
    assert b.to_list() == [x for x in 0..k / 3] and mid.to_list() == [x for x in k / 3..k]
    b.join(mid)
    check_bag_against_counts(b, cnt)
    let top = b.split_at(k - 10)
    assert top.to_list() == [x for x in k - 10..k] and b.len == k - 10
    b.join(top)
    check_bag_against_counts(b, cnt)

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
            assert m.keys.collect() == [e.0 for e in have] and m.values.collect() == [e.1 for e in have]
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
    lo_inc: Bool
    hi: ?K
    hi_inc: Bool

    /// The smallest key in the range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).first == None`
    /// ties: `let b = SortedBag.of([1, 3, 3]); b.range(2..5).first == Some(3)`
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(2..).first == Some(3)`
    @get pub fn first(self) -> ?K = bag.range_first(lo, lo_inc, hi, hi_inc)

    /// The largest key in the range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).last == None`
    /// ties: `let b = SortedBag.of([3, 3, 5]); b.range(2..5).last == Some(3)`
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..=6).last == Some(3)`
    @get pub fn last(self) -> ?K = bag.range_last(lo, lo_inc, hi, hi_inc)

    /// The number of keys in the range, counting copies.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).len == 0`
    /// traps: never
    /// example: `SortedBag.of([1, 3, 3, 7]).range(3..=3).len == 2`
    @get pub fn len(self) -> Int = bag.range_len(lo, lo_inc, hi, hi_inc)

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
    pub fn each(self) -> yields K = bag.range_each(lo, lo_inc, hi, hi_inc)

    /// The keys in descending order, copies included.
    ///
    /// cost: \(O(\log n + k)\) expected for `k` keys visited
    /// empty: `let b = SortedBag.of([1, 5]); b.range(2..5).rev.collect() == []`
    /// order: descending
    /// traps: never
    /// example: `SortedSet.of([1, 3, 7]).range(..7).rev.collect() == [3, 1]`
    @get pub fn rev(self) -> yields K = bag.range_each_rev(lo, lo_inc, hi, hi_inc)

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
