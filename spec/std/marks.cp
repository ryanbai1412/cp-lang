//! Opt-in generation-stamped mark table: `add`, `in`, `remove`, and `clear` are all O(1).
//!
//! For sparse resets prefer `[D]Bool` with a touched list; import `std.marks.Marks<D>` when
//! repeated full clears inside a loop would otherwise cost O(T * N). Each `clear()` bumps a
//! 32-bit generation counter instead of touching the underlying array, resetting the table
//! only if the counter reaches its 32-bit ceiling.

use std.interval.{IntervalMap}

const MAX_GEN = 2_000_000_000

/// A visited/mark set over a domain `D` with O(1) `add`, `remove`, `contains` (`v in m`),
/// and O(1) `clear()`. Memory is one `[D]0..=2_000_000_000` array (4 bytes per element).
pub struct Marks<D>:
    stamp: [D]0..=2_000_000_000
    gen: Int
    cnt: Int

    /// An empty mark table over `D`. O(D.K) once; subsequent `clear()` calls are O(1).
    pub fn new() -> Marks<D> = Marks([D] _ => 0, 1, 0)

    /// Mark `v` in the current round. Returns `true` iff `v` was not already marked. O(1).
    pub fn add(mut self, v: D) -> Bool:
        if stamp[v] == gen: return false
        stamp[v] = gen
        cnt += 1
        true

    /// Unmark `v` in the current round. Returns `true` iff `v` was marked. O(1).
    pub fn remove(mut self, v: D) -> Bool:
        if stamp[v] != gen: return false
        stamp[v] = 0
        cnt -= 1
        true

    /// Is `v` marked in the current round? Also `v in m`. O(1).
    pub fn contains(self, v: D) -> Bool = stamp[v] == gen

    /// Clear every mark in O(1) without touching the array (resets the table on wraparound).
    pub fn clear(mut self):
        if gen == MAX_GEN:
            stamp.fill(0)
            gen = 1
        else:
            gen += 1
        cnt = 0

    /// How many positions are marked in the current round. O(1).
    pub @get fn len(self) -> Int = cnt

    /// Whether no positions are currently marked. O(1).
    pub @get fn is_empty(self) -> Bool = cnt == 0

/// Epoch-stamped map over domain `D` with O(1) `get`, `set`, `contains`, `remove`, and `clear()`.
pub struct MarkMap<D, T>:
    stamp: [D]0..=2_000_000_000
    vals: [D]T
    default_val: T
    gen: Int
    cnt: Int

    /// Create an empty `MarkMap<D, T>` backed by `default_val` for unset slots. O(D.K).
    pub fn new(default_val: T) -> MarkMap<D, T>:
        MarkMap([D] _ => 0, [D] _ => default_val, default_val, 1, 0)

    /// Number of active entries in the current round. O(1).
    pub @get fn len(self) -> Int = cnt

    /// Whether no entries are active in the current round. O(1).
    pub @get fn is_empty(self) -> Bool = cnt == 0

    /// Whether `v` is present in the current round. O(1).
    pub fn contains(self, v: D) -> Bool = stamp[v] == gen

    /// Current value at `v`, or `none` if absent in the current round. O(1).
    pub fn get(self, v: D) -> ?T = if stamp[v] == gen: vals[v] else: none

    /// Set `v` to `val` in the current round. O(1).
    pub fn set(mut self, v: D, val: T):
        if stamp[v] != gen:
            stamp[v] = gen
            cnt += 1
        vals[v] = val

    /// Remove `v` from the current round; returns its previous value or `none`. O(1).
    pub fn remove(mut self, v: D) -> ?T:
        if stamp[v] != gen: return none
        stamp[v] = 0
        cnt -= 1
        vals[v]

    /// Clear every entry in O(1) without touching the value array.
    pub fn clear(mut self):
        if gen == MAX_GEN:
            stamp.fill(0)
            gen = 1
        else:
            gen += 1
        cnt = 0

/// Online value-to-id interner assigning contiguous `0..len` IDs to distinct keys `K`.
/// `id`, `get`, and `key` are O(1) expected; `len`, `is_empty`, `clear` are O(1).
pub struct Interner<K>:
    to_id: HashMap<K, Int>
    keys: List<K>

    /// Create an empty `Interner<K>`. O(1).
    pub fn new() -> Interner<K> = Interner(HashMap<K, Int>(), [])

    /// Number of distinct interned keys. O(1).
    pub @get fn len(self) -> Int = keys.len

    /// Whether no keys have been interned. O(1).
    pub @get fn is_empty(self) -> Bool = keys.is_empty

    /// Remove all interned keys. O(1).
    pub fn clear(mut self):
        to_id.clear()
        keys = []

    /// Return the ID of `k`, assigning the next contiguous ID if `k` is new. O(1) expected.
    pub fn id(mut self, k: K) -> Int:
        if let i = to_id.get(k): return i
        let i = keys.len
        keys.push(k)
        to_id[k] = i
        i

    /// Look up the ID of `k` without inserting, or `none` if unseen. O(1) expected.
    pub fn get(self, k: K) -> ?Int = to_id.get(k)

    /// Recover the key assigned to `i`. O(1).
    pub fn key(self, i: Int) -> K = keys[i]

    /// Whether `k` has already been interned. O(1) expected.
    pub fn contains(self, k: K) -> Bool = todo

    /// Preallocate storage for at least `cap` distinct keys. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

/// Briggs-Torczon sparse set over domain `D`.
/// O(1) `add`, `remove`, `contains`, `clear` AND O(`len`) iteration over active elements.
pub struct SparseSet<D>:
    _len: Int

    /// Create an empty `SparseSet<D>`. O(D.K).
    pub fn new() -> SparseSet<D> = todo

    /// Number of active elements. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no elements are active. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether `v` is currently in the set. O(1).
    pub fn contains(self, v: D) -> Bool = todo

    /// Insert `v`; returns true if `v` was not already present. O(1).
    pub fn add(mut self, v: D) -> Bool = todo

    /// Remove `v`; returns true if `v` was present. O(1).
    pub fn remove(mut self, v: D) -> Bool = todo

    /// Clear all elements in O(1) without touching the sparse table.
    pub fn clear(mut self) = todo

    /// Yield all currently active elements in arbitrary order. O(`len`).
    pub fn each(self) -> yields D = todo

/// Compact 4-byte typed handle into an `Arena<T>`.
pub struct Id<T>(raw: 0..=2_000_000_000)

/// Contiguous pool allocator returning compact 4-byte `Id<T>` handles (for user trees, tries, and persistent nodes).
pub struct Arena<T>:
    _xs: List<T>

    /// Create an empty `Arena<T>` with optional initial capacity `cap`. O(cap).
    pub fn new(cap: Int = 0) -> Arena<T> = todo

    /// Number of allocated nodes. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no nodes have been allocated. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Allocate `x` and return its compact handle. O(1) amortized.
    pub fn alloc(mut self, x: T) -> Id<T> = todo

    /// Read the node at `id` (W8 `Indexed` backing `arena[id]`). O(1).
    pub fn get(self, id: Id<T>) -> T = todo

    /// Overwrite the node at `id` (W8 `IndexSet` backing `arena[id] = x`). O(1).
    pub fn set(mut self, id: Id<T>, x: T) = todo

    /// Reserve backing storage for at least `cap` nodes. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Reset the arena to empty (invalidates existing handles). O(1).
    pub fn clear(mut self) = todo

@test
fn mark_map_and_interner():
    domain V = 0..10
    var mm = MarkMap<V, Int>(0)
    assert mm.is_empty and mm.get(V(2)) == none
    mm.set(V(2), 7)
    mm.set(V(5), -3)
    assert mm.len == 2 and mm.contains(V(2)) and mm.get(V(2)) == 7 and mm.get(V(5)) == -3
    assert mm.remove(V(2)) == 7 and mm.remove(V(2)) == none and mm.len == 1
    mm.clear()
    assert mm.is_empty and mm.get(V(5)) == none
    mm.set(V(5), 99)
    assert mm.get(V(5)) == 99 and mm.len == 1

    var intern = Interner<Str>()
    assert intern.is_empty and intern.get("a") == none
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
    domain V = 0..16
    var m = Marks<V>()
    var seen: [V]Bool = false
    for _ in 0..40:
        for _ in 0..30:
            let v = V.nth(rand(0, 15))
            let op = rand(0, 3)
            if op <= 1:
                let want = not seen[v]
                seen[v] = true
                assert m.add(v) == want
            elif op == 2:
                let want = seen[v]
                seen[v] = false
                assert m.remove(v) == want
            else:
                assert (v in m) == seen[v] and m.contains(v) == seen[v]
        assert m.len == count(seen[v] for v in V)
        assert m.is_empty == (m.len == 0)
        for v in V: assert (v in m) == seen[v]
        m.clear()
        seen.fill(false)
        assert m.is_empty and m.len == 0

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
    // Default no-op callback + explicit callback from another module.
    im.assign(P(2)..=P(5), 3)
    assert im.runs == 3 and im.at(P(2)) == 3 and im.at(P(6)) == 0
    var total_len = 0
    var sum_old = 0
    fn record(r: Range<P>, old: Int):
        total_len += r.len
        sum_old += r.len * old
    im.assign(P(4)..=P(7), 3, on_old: record)
    assert total_len == 4 and sum_old == 6
    assert im.runs == 3 and im.run_at(P(5)).range.len == 6
