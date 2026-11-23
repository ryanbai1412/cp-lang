//! Opt-in generation-stamped mark table: `add`, `in`, `remove`, and `clear` are all \(O(1)\).
//!
//! For sparse resets prefer `[D]Bool` with a touched list; import `std.marks.Marks<D>` when
//! repeated full clears inside a loop would otherwise cost \(O(T \cdot N)\). Each `clear()` bumps a
//! 32-bit generation counter instead of touching the underlying array, resetting the table
//! only if the counter reaches its 32-bit ceiling.

use std.interval.{IntervalMap}

const MAX_GEN = 2_000_000_000

/// A visited/mark set over a domain `D` with \(O(1)\) `add`, `remove`, `contains` (`v in m`),
/// and \(O(1)\) `clear()`. Memory is one `[D]0..=2_000_000_000` array (4 bytes per element).
pub struct Marks<D>:
    stamp: [D]0..=2_000_000_000
    var gen: Int
    cnt: Int

    /// An empty mark set over `D`, with its stamp array allocated once.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: an empty `D` gives a set that can never hold a mark
    /// traps: never
    /// example: `Marks<0..5>().is_empty`
    pub fn new() -> Marks<D> = Marks([D] _ => 0, 1, 0)

    /// Mark `v`. Returns `true` when `v` was unmarked, `false` when it was already marked (and
    /// nothing changes).
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = Marks<0..5>(); m.add(2) and not m.add(2)`
    pub fn add(mut self, v: D) -> Bool:
        if stamp[v] == gen: return false
        stamp[v] = gen
        cnt += 1
        true

    /// Unmark `v`. Returns `true` when `v` was marked, `false` when it was not (and nothing changes).
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = Marks<0..5>(); m.add(2); m.remove(2) and not m.remove(2)`
    pub fn remove(mut self, v: D) -> Bool:
        if stamp[v] != gen: return false
        stamp[v] = 0
        cnt -= 1
        true

    /// Whether `v` is marked; also spelled `v in m`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = Marks<0..5>(); m.add(1); m.contains(1) and not m.contains(0)`
    pub fn contains(self, v: D) -> Bool = stamp[v] == gen

    /// Unmark everything by moving to a new generation, without touching the stamp array. Once the
    /// generation counter reaches `2_000_000_000` the array is reset to zero, so the cost stays \(O(1)\)
    /// amortized over the calls.
    ///
    /// cost: \(O(1)\) amortized; \(O(\lvert D \rvert)\) on the call that wraps the counter
    /// traps: never
    /// example: `var m = Marks<0..5>(); m.add(1); m.add(3); m.clear(); m.is_empty and not (1 in m)`
    pub fn clear(mut self):
        if gen == MAX_GEN:
            stamp.fill(0)
            gen = 1
        else:
            gen += 1
        cnt = 0

    /// The number of marked positions.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = Marks<0..5>(); m.add(1); m.add(3); m.add(1); m.len == 2`
    @get pub fn len(self) -> Int:
        cnt

    /// Whether no position is marked; `len == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = Marks<0..5>(); m.add(0); m.remove(0); m.is_empty`
    @get pub fn is_empty(self) -> Bool:
        cnt == 0

/// Epoch-stamped map over domain `D` with \(O(1)\) `get`, `set`, `contains`, `remove`, and `clear()`.
pub struct MarkMap<D, T>:
    stamp: [D]0..=2_000_000_000
    vals: [D]T
    default_val: T
    gen: Int
    cnt: Int

    /// An empty map over `D`. `default_val` fills the value array and is never returned: an absent
    /// key reads as `None`.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: an empty `D` gives a map that can never hold an entry
    /// traps: never
    /// example: `MarkMap<0..5, Int>(0).is_empty`
    pub fn new(default_val: T) -> MarkMap<D, T>:
        MarkMap([D] _ => 0, [D] _ => default_val, default_val, 1, 0)

    /// The number of keys present.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(1, 7); m.set(1, 8); m.set(4, 0); m.len == 2`
    @get pub fn len(self) -> Int:
        cnt

    /// Whether no key is present; `len == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(2, 1); m.remove(2); m.is_empty`
    @get pub fn is_empty(self) -> Bool:
        cnt == 0

    /// Whether `v` has a value. A key set to `default_val` is present.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(2, 0); m.contains(2) and not m.contains(3)`
    pub fn contains(self, v: D) -> Bool = stamp[v] == gen

    /// The value at `v`, or `None` when `v` is absent.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(2, 9); m.get(2) == Some(9) and m.get(3) == None`
    pub fn get(self, v: D) -> ?T = if stamp[v] == gen: vals[v] else: None

    /// Give `v` the value `val`, adding `v` when it is absent and overwriting it otherwise.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(2, 9); m.set(2, 4); m.get(2) == Some(4)`
    pub fn set(mut self, v: D, val: T):
        if stamp[v] != gen:
            stamp[v] = gen
            cnt += 1
        vals[v] = val

    /// Remove `v` and return the value it had, or `None` (and change nothing) when it was absent.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(2, 9); m.remove(2) == Some(9) and m.remove(2) == None`
    pub fn remove(mut self, v: D) -> ?T:
        if stamp[v] != gen: return None
        stamp[v] = 0
        cnt -= 1
        vals[v]

    /// Remove every key by moving to a new generation, without touching the value array; the
    /// arrays are reset when the counter reaches `2_000_000_000`, as for `Marks.clear`.
    ///
    /// cost: \(O(1)\) amortized; \(O(\lvert D \rvert)\) on the call that wraps the counter
    /// traps: never
    /// example: `var m = MarkMap<0..5, Int>(0); m.set(2, 9); m.clear(); m.get(2) == None and m.is_empty`
    pub fn clear(mut self):
        if gen == MAX_GEN:
            stamp.fill(0)
            gen = 1
        else:
            gen += 1
        cnt = 0

/// Online value-to-id interner assigning contiguous `0..len` IDs to distinct keys `K`.
/// `id`, `get`, and `key` are \(O(1)\) expected; `len`, `is_empty`, `clear` are \(O(1)\).
pub struct Interner<K>:
    to_id: HashMap<K, Int>
    keys: List<K>

    /// An empty interner.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Interner<Str>().is_empty`
    pub fn new() -> Interner<K> = Interner(HashMap<K, Int>(), [])

    /// The number of distinct keys interned so far; ids are exactly `0..len`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = Interner<Str>(); t.id("a"); t.id("b"); t.id("a"); t.len == 2`
    @get pub fn len(self) -> Int:
        keys.len

    /// Whether no key has been interned; `len == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = Interner<Str>(); t.id("a"); t.clear(); t.is_empty`
    @get pub fn is_empty(self) -> Bool:
        keys.len == 0

    /// Forget every key, so the next new key gets id `0` again.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = Interner<Str>(); t.id("a"); t.id("b"); t.clear(); t.id("b") == 0`
    pub fn clear(mut self):
        to_id.clear()
        keys = []

    /// The id of `k`. A new key gets the next id, `len`, so ids follow first-seen order.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var t = Interner<Str>(); t.id("x") == 0 and t.id("y") == 1 and t.id("x") == 0`
    pub fn id(mut self, k: K) -> Int:
        if let i = to_id.get(k): return i
        let i = keys.len
        keys.push(k)
        to_id[k] = i
        i

    /// The id of `k`, or `None` when `k` has not been interned; never adds a key.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var t = Interner<Str>(); t.id("x"); t.get("x") == Some(0) and t.get("y") == None and t.len == 1`
    pub fn get(self, k: K) -> ?Int = to_id.get(k)

    /// The key whose id is `i`; the inverse of `id`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is outside `0..len`
    /// example: `var t = Interner<Str>(); t.id("x"); t.id("y"); t.key(1) == "y"`
    pub fn key(self, i: Int) -> K = keys[i]

    /// Whether `k` has been interned; `get(k) != None`.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var t = Interner<Int>(); t.id(40); t.contains(40) and not t.contains(41)`
    pub fn contains(self, k: K) -> Bool:
        to_id.get(k) != None

    /// Allocate room for at least `cap` distinct keys, so the next `cap - len` new keys do not
    /// reallocate. Changes no ids.
    ///
    /// cost: \(O(\mathit{cap})\)
    /// traps: when `cap < 0`
    /// example: `var t = Interner<Int>(); t.reserve(100); t.id(5); t.len == 1`
    pub fn reserve(mut self, cap: Int):
        assert cap >= 0
        keys.reserve(cap)

/// Briggs-Torczon sparse set over domain `D`.
/// \(O(1)\) `add`, `remove`, `contains`, `clear` AND O(`len`) iteration over active elements.
pub struct SparseSet<D>:
    _len: Int
    dense: List<D>
    pos: [D]Int

    /// An empty sparse set over `D`.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: an empty `D` gives a set that can never hold an element
    /// traps: never
    /// example: `SparseSet<0..5>().is_empty`
    pub fn new() -> SparseSet<D>:
        SparseSet(0, [], [D] _ => -1)

    /// The number of elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = SparseSet<0..5>(); s.add(1); s.add(4); s.add(1); s.len == 2`
    @get pub fn len(self) -> Int:
        _len

    /// Whether the set has no elements; `len == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = SparseSet<0..5>(); s.add(1); s.remove(1); s.is_empty`
    @get pub fn is_empty(self) -> Bool:
        _len == 0

    /// Whether `v` is in the set.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var s = SparseSet<0..5>(); s.add(3); s.contains(3) and not s.contains(2)`
    pub fn contains(self, v: D) -> Bool:
        let p = pos[v]
        p >= 0 and p < _len and dense[p] == v

    /// Add `v` at the end of the dense order. Returns `true` when `v` was absent, `false` when it
    /// was present (and nothing changes).
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var s = SparseSet<0..5>(); s.add(3) and not s.add(3)`
    pub fn add(mut self, v: D) -> Bool:
        if self.contains(v): return false
        if _len < dense.len: dense[_len] = v
        else: dense.push(v)
        pos[v] = _len
        _len += 1
        true

    /// Remove `v`, moving the last element of the dense order into its slot. Returns `true` when `v`
    /// was present, `false` when it was absent (and nothing changes).
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is outside `D`
    /// example: `var s = SparseSet<0..5>(); s.add(3); s.add(1); s.add(4); s.remove(3); s.each().collect() == [4, 1]`
    pub fn remove(mut self, v: D) -> Bool:
        if not self.contains(v): return false
        let p = pos[v]
        let last = dense[_len - 1]
        dense[p] = last
        pos[last] = p
        _len -= 1
        true

    /// Remove every element by resetting the length; the sparse table is not touched.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = SparseSet<0..5>(); s.add(3); s.clear(); s.is_empty and not s.contains(3)`
    pub fn clear(mut self):
        _len = 0

    /// Every element, in dense order: insertion order, except that each `remove` moved the then-last
    /// element into the removed one's slot.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: yields nothing for an empty set
    /// order: dense order (insertion order adjusted by removals, as above)
    /// traps: never
    /// example: `var s = SparseSet<0..5>(); s.add(2); s.add(0); s.add(4); s.each().collect() == [2, 0, 4]`
    pub fn each(self) -> yields D:
        for i in 0.._len: yield dense[i]

/// Compact 4-byte typed handle into an `Arena<T>`.
pub struct Id<T>(raw: 0..=2_000_000_000)

/// Contiguous pool allocator returning compact 4-byte `Id<T>` handles (for user trees, tries, and persistent nodes).
pub struct Arena<T>:
    _xs: List<T>

    /// An empty arena with room for `cap` nodes before it reallocates.
    ///
    /// cost: \(O(\mathit{cap})\)
    /// traps: when `cap < 0`
    /// example: `Arena<Int>(cap: 8).is_empty`
    pub fn new(cap: Int = 0) -> Arena<T>:
        var xs: List<T> = []
        xs.reserve(cap)
        Arena(xs)

    /// The number of nodes allocated since creation or the last `clear`; valid handles have raw
    /// values `0..len`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var a = Arena<Int>(); a.alloc(5); a.alloc(6); a.len == 2`
    @get pub fn len(self) -> Int:
        _xs.len

    /// Whether no node is allocated; `len == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var a = Arena<Int>(); a.alloc(5); a.clear(); a.is_empty`
    @get pub fn is_empty(self) -> Bool:
        _xs.len == 0

    /// Store `x` as a new node and return its handle; handles are issued in order `0, 1, 2, ...`.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when the arena already holds `2_000_000_001` nodes
    /// example: `var a = Arena<Str>(); a.alloc("p"); a.alloc("q") == Id<Str>(1)`
    pub fn alloc(mut self, x: T) -> Id<T>:
        _xs.push(x)
        Id(_xs.len - 1)

    /// The node behind `id`; also `arena[id]`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `id` is not below `len`. A handle issued before a `clear` is not detected while
    /// it is below `len`; it reads whatever node now holds that slot
    /// example: `var a = Arena<Int>(); let i = a.alloc(5); a.alloc(6); a.get(i) == 5`
    pub fn get(self, id: Id<T>) -> T:
        _xs[id.raw]

    /// Overwrite the node behind `id` with `x`; also `arena[id] = x`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `id` is not below `len` (stale handles as in `get`)
    /// example: `var a = Arena<Int>(); let i = a.alloc(5); a.set(i, 9); a[i] == 9`
    pub fn set(mut self, id: Id<T>, x: T):
        _xs[id.raw] = x

    /// Allocate room for at least `cap` nodes in total; changes no handle or node.
    ///
    /// cost: \(O(\mathit{cap})\)
    /// traps: when `cap < 0`
    /// example: `var a = Arena<Int>(); a.reserve(100); a.alloc(1); a.len == 1`
    pub fn reserve(mut self, cap: Int):
        _xs.reserve(cap)

    /// Drop every node, so the next `alloc` returns handle `0` again. Every handle issued before is
    /// invalidated.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var a = Arena<Int>(); a.alloc(5); a.clear(); a.alloc(7) == Id<Int>(0)`
    pub fn clear(mut self):
        _xs.clear()

@test
fn mark_map_and_interner():
    domain V = 0..10
    var mm = MarkMap<V, Int>(0)
    assert mm.is_empty and mm.get(V(2)) == None
    mm.set(V(2), 7)
    mm.set(V(5), -3)
    assert mm.len == 2 and mm.contains(V(2)) and mm.get(V(2)) == 7 and mm.get(V(5)) == -3
    assert mm.remove(V(2)) == 7 and mm.remove(V(2)) == None and mm.len == 1
    mm.clear()
    assert mm.is_empty and mm.get(V(5)) == None
    mm.set(V(5), 99)
    assert mm.get(V(5)) == 99 and mm.len == 1

    var intern = Interner<Str>()
    assert intern.is_empty and intern.get("a") == None
    assert intern.id("a") == 0 and intern.id("b") == 1 and intern.id("a") == 0
    assert intern.len == 2 and intern.get("b") == 1 and intern.key(0) == "a" and intern.key(1) == "b"
    intern.clear()
    assert intern.is_empty and intern.id("c") == 0

@test
fn marks_basic_operations_and_clear():
    domain V = 1..=8
    var m = Marks<V>()
    assert m.is_empty and m.len == 0
    assert m.add(V(3)) and not m.add(V(3))
    assert m.add(V(7)) and m.len == 2 and not m.is_empty
    assert V(3) in m and V(7) in m and V(1) not in m
    assert m.remove(V(3)) and not m.remove(V(3))
    assert V(3) not in m and m.len == 1
    m.clear()
    assert m.is_empty and m.len == 0 and V(7) not in m
    // Re-adding after clear succeeds as fresh
    assert m.add(V(7)) and V(7) in m and m.len == 1

@test
fn marks_matches_bool_array_across_many_rounds():
    domain V = 0..30
    var m = Marks<V>()
    var mm = MarkMap<V, Int>(0)
    for _ in 0..40:
        var plain: [V]Bool = false
        var vals: [V]Int = 0
        for _ in 0..60:
            let v = V(rand(0, 29))
            let x = rand(1, 9)
            if rand(0, 2) == 0:
                assert m.remove(v) == plain[v]
                assert mm.remove(v) == (if plain[v]: Some(vals[v]) else: None)
                plain[v] = false
            else:
                assert m.add(v) == (not plain[v])
                mm.set(v, x)
                plain[v] = true
                vals[v] = x
        var count = 0
        for v in V:
            assert (v in m) == plain[v]
            assert mm.get(v) == (if plain[v]: Some(vals[v]) else: None)
            if plain[v]: count += 1
        assert m.len == count
        m.clear()
        mm.clear()

@test
fn marks_over_char_and_one_based_domains():
    domain Letter = 'a'..='z'
    var seen = Marks<Letter>()
    assert seen.add(Letter('a')) and seen.add(Letter('z')) and seen.len == 2
    assert Letter('a') in seen and Letter('m') not in seen
    seen.clear()
    assert Letter('a') not in seen and seen.is_empty
    // Repeated O(1) clears over a larger domain
    domain Big = 1..=200
    var big = Marks<Big>()
    for round in 1..=20:
        let v = Big(round * 7 % 200 + 1)
        assert big.add(v) and not big.add(v) and big.len == 1
        big.clear()
        assert big.is_empty and v not in big

@test
fn marks_wraps_cleanly_at_32bit_ceiling():
    domain V = 0..6
    var m = Marks<V>()
    m.gen = MAX_GEN
    assert m.add(V(2)) and m.add(V(5)) and m.len == 2
    assert V(2) in m and V(5) in m
    // Clearing at MAX_GEN resets the backing stamps and restarts generation at 1
    m.clear()
    assert m.is_empty and m.len == 0 and m.gen == 1
    for v in V: assert v not in m and m.stamp[v] == 0
    assert m.add(V(2)) and V(2) in m and V(5) not in m and m.len == 1

@test
fn interval_map_cross_module_assign():
    domain P = 1..=8
    var im = IntervalMap<P, Int>.new(0)
    im.assign(P(2)..=P(5), 3)
    assert im.runs == 3 and im.get(P(2)) == 3 and im.get(P(6)) == 0
    var total_len = 0
    var sum_old = 0
    im.assign(P(4)..=P(7), 3, on_old: (r, old) =>
        total_len += r.len
        sum_old += r.len * old
    )
    assert total_len == 4 and sum_old == 6
    assert im.runs == 3 and im.run_at(P(5)).range.len == 6
