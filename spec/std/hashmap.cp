//! Open-addressing hash map for `Int` keys with splitmix64 salting.
//!
//! `FlatMap<V>` stores keys, values, and occupancy states in flat parallel arrays with
//! linear probing and power-of-two capacity, avoiding per-node heap allocations when
//! built-in `HashMap<Int, V>` overhead matters.
use std.math.{splitmix64}

// Slot states: 0 = empty, 1 = occupied; `remove` uses backward-shift deletion (no tombstones).
/// Flat open-addressing hash map from `Int` keys to `V`, salted with `splitmix64`.
/// `get`, `contains`, `set`, `remove`, and `get_or_insert` are O(1) expected.
pub struct FlatMap<V>:
    keys: List<Int>
    vals: List<V>
    state: List<Int>
    default_val: V
    salt: U64
    live: Int
    used: Int

    /// [rename] Create an empty `FlatMap<V>` sized for at least `cap` entries (replaces `with_capacity`).
    ///
    /// `default_val` fills the value slots that hold no entry; no method returns it, so any
    /// value of `V` works. The table is a power of two at least twice `cap`, so `cap` entries
    /// fit without a rehash; a `cap` below 1 gives the smallest table.
    ///
    /// cost: O(cap)
    /// traps: never
    /// example: `var m: FlatMap<Int> = FlatMap(0); m.len == 0`
    pub fn new(default_val: V, cap: Int = 16) -> FlatMap<V> = todo

    /// Number of live key-value pairs.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(1, 5); m.set(1, 6); m.len == 1`
    @get pub fn len(self) -> Int = todo

    /// Whether the map has no live entries.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `FlatMap(0).is_empty == true`
    @get pub fn is_empty(self) -> Bool = todo

    /// Remove all entries while keeping allocated capacity.
    ///
    /// cost: O(capacity)
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(1, 5); m.clear(); m.get(1) == None`
    pub fn clear(mut self):
        for i in 0..state.len: state[i] = 0
        live = 0
        used = 0

    fn slot(self, k: Int) -> Int:
        let h = splitmix64(U64(k) ^ salt)
        Int(h & U64(state.len - 1))

    fn rehash(mut self, new_cap: Int):
        let old_k = keys
        let old_v = vals
        let old_s = state
        keys = List.filled(new_cap, 0)
        vals = List.filled(new_cap, default_val)
        state = List.filled(new_cap, 0)
        live = 0
        used = 0
        for i in 0..old_s.len:
            if old_s[i] == 1:
                self.set(old_k[i], old_v[i])

    /// Look up `k`, returning `None` if absent.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(5, 50); m.get(5) == Some(50) and m.get(6) == None`
    pub fn get(self, k: Int) -> ?V:
        let mask = state.len - 1
        var i = self.slot(k)
        unchecked:
            while state[i] != 0:
                if state[i] == 1 and keys[i] == k: return vals[i]
                i = (i + 1) & mask
        None

    /// Whether `k` is currently present.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(-3, 1); m.contains(-3) and not m.contains(3)`
    pub fn contains(self, k: Int) -> Bool = self.get(k) != None

    /// Insert or overwrite `k -> v`.
    ///
    /// When the table would pass half full it doubles and every entry is rehashed.
    ///
    /// cost: O(1) expected amortized
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(1, 5); m.set(1, 6); m.get(1) == Some(6)`
    pub fn set(mut self, k: Int, v: V):
        if (used + 1) * 10 >= state.len * 7:
            let next_cap = if live * 4 < state.len: state.len else: state.len * 2
            self.rehash(next_cap)
        let mask = state.len - 1
        var i = self.slot(k)
        var tomb = -1
        unchecked:
            while state[i] != 0:
                if state[i] == 1 and keys[i] == k:
                    vals[i] = v
                    return
                if state[i] == 2 and tomb < 0: tomb = i
                i = (i + 1) & mask
            let dst = if tomb >= 0: tomb else: i
            if state[dst] == 0: used += 1
            keys[dst] = k
            vals[dst] = v
            state[dst] = 1
        live += 1

    /// Return the value for `k`, inserting `v` first if `k` is absent.
    ///
    /// A present `k` keeps its value; `v` is then ignored.
    ///
    /// cost: O(1) expected amortized
    /// traps: never
    /// example: `var m = FlatMap(0); m.get_or_insert(4, 7) == 7 and m.get_or_insert(4, 9) == 7`
    pub fn get_or_insert(mut self, k: Int, v: V) -> V:
        if let cur = self.get(k): return cur
        self.set(k, v)
        v

    /// Remove `k` if present; returns its old value or `None`.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(2, 9); m.remove(2) == Some(9) and m.remove(2) == None`
    pub fn remove(mut self, k: Int) -> ?V:
        let mask = state.len - 1
        var i = self.slot(k)
        unchecked:
            while state[i] != 0:
                if state[i] == 1 and keys[i] == k:
                    state[i] = 2
                    live -= 1
                    return vals[i]
                i = (i + 1) & mask
        None

    /// Reserve space for at least `cap` entries without rehashing.
    ///
    /// Entries are kept; a `cap` the table already fits does nothing.
    ///
    /// cost: O(cap) when the table grows, O(1) otherwise
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(1, 2); m.reserve(1000); m.get(1) == Some(2)`
    pub fn reserve(mut self, cap: Int) = todo

    /// Move the map's contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(1, 2); let t = m.take(); m.is_empty and t.get(1) == Some(2)`
    pub fn take(mut self) -> FlatMap<V> = todo

    /// O(1) to make. Yields every live `(key, val)` pair.
    ///
    /// The order is the table's slot order, which depends on the per-map salt: it is fixed
    /// while the map is unchanged and agrees with `keys` and `values`, but no program should
    /// depend on it. Sort when the order matters.
    ///
    /// cost: O(1) to make, O(capacity) to walk
    /// empty: yields nothing
    /// order: unspecified slot order, the same as `keys` and `values` while unchanged
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(2, 20); m.set(1, 10); m.items.sorted() == [(1, 10), (2, 20)]`
    @get pub fn items(self) -> yields (Int, V) = todo

    ///
    ///
    /// cost: O(1) to make, O(capacity) to walk
    /// empty: yields nothing
    /// order: unspecified slot order, the same as `items`
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(2, 20); m.set(1, 10); m.keys.sorted() == [1, 2]`
    @get pub fn keys(self) -> yields Int = todo

    ///
    ///
    /// cost: O(1) to make, O(capacity) to walk
    /// empty: yields nothing
    /// order: unspecified slot order, the same as `items`
    /// traps: never
    /// example: `var m = FlatMap(0); m.set(2, 20); m.set(1, 10); m.values.sorted() == [10, 20]`
    @get pub fn values(self) -> yields V = todo

    /// Merge all entries from `other` into `self` (emptying `other`), combining collisions with `join` or overwriting.
    ///
    /// For a key in both maps the result is `join(self's value, other's value)`, or `other`'s
    /// value when `join` is `None`. The smaller map is drained into the larger, so the
    /// argument order of `join` and the overwrite rule hold whichever side is moved.
    ///
    /// cost: O(min(|self|, |other|)) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never
    /// example: `var a = FlatMap(0); a.set(1, 10); var b = FlatMap(0); b.set(1, 5); b.set(2, 7); a.absorb(mut b, join: (x, y) => x + y); a.get(1) == Some(15) and a.get(2) == Some(7) and b.is_empty`
    /// example: `var a = FlatMap(0); a.set(1, 10); var b = FlatMap(0); b.set(1, 5); a.absorb(mut b); a.get(1) == Some(5)`
    pub fn absorb(mut self, mut other: FlatMap<V>, join: ?((V, V) -> V) = None) = todo

/// Default-valued frequency / weight map over hashable keys `K`. `Counter<K>` counts with `Int`
/// (`V = Int`) and is a `Track<K>` (`std.window`): `push(k)` adds 1, `pop(k)` takes 1 away.
/// `c[k]` reads `init` without inserting; `add`, `get`, `set`, `remove`, and `contains` are O(1) expected.
///
/// Invariant: no stored entry equals `init`. A write that leaves a key at `init` (`add`, `set`,
/// `absorb`) removes it, so `len` is the number of keys whose value differs from `init` and
/// `contains(k)` is `get(k) != init`.
pub struct Counter<K, V = Int>:
    _len: Int

    /// Create an empty `Counter<K, V>` whose missing keys read as `init`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var c: Counter<Char, Int> = Counter(0); c.get('z') == 0 and c.len == 0`
    pub fn new(init: V) -> Counter<K, V> = todo

    /// Count occurrences of each element in `xs`.
    ///
    /// The result has `init` `0`, so a key that never occurs reads `0`.
    ///
    /// cost: O(n) expected
    /// empty: an empty counter
    /// traps: never
    /// example: `let c = Counter.of(['a', 'b', 'a']); c.get('a') == 2 and c.get('c') == 0`
    pub fn of(xs: List<K>) -> Counter<K, Int> = todo

    /// Number of keys with a stored entry.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Counter.of([3, 1, 3]).len == 2`
    @get pub fn len(self) -> Int = todo

    /// Whether no keys are stored.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Counter.of([1]).is_empty == false`
    @get pub fn is_empty(self) -> Bool = todo

    /// Value stored for `k`, or `init` if absent (does not insert).
    ///
    /// `c[k]` calls this.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `let c = Counter.of([5, 5]); c[5] == 2 and c[6] == 0 and c.len == 1`
    pub fn get(self, k: K) -> V = todo

    /// Whether `k` currently has a stored entry.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `let c = Counter.of([5]); c.contains(5) and not c.contains(6)`
    pub fn contains(self, k: K) -> Bool = todo

    /// Set `k` to `v` (W8 `IndexSet` backing `c[k] = v`).
    ///
    /// Setting a key to `init` removes it.
    ///
    /// cost: O(1) expected amortized
    /// traps: never
    /// example: `var c = Counter.of(['a']); c['a'] = 4; c['a'] == 4`
    /// example: `var c = Counter.of(['a']); c['a'] = 0; c.contains('a') == false`
    pub fn set(mut self, k: K, v: V) = todo

    /// Add `delta` to `k`'s value, removing `k` if the result equals `init`.
    ///
    /// An absent key starts from `init`. `c[k] += d` is `c.add(k, d)`.
    ///
    /// cost: O(1) expected amortized
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `var c: Counter<Char, Int> = Counter(0); c.add('x', 2); c['x'] == 2`
    /// example: `var c: Counter<Char, Int> = Counter(0); c.add('x', 2); c.add('x', -2); c.len == 0`
    pub fn add(mut self, k: K, delta: V) = todo

    /// Remove `k` and return its previous value, or `None` if absent.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var c = Counter.of(['a', 'a']); c.remove('a') == Some(2) and c.remove('a') == None`
    pub fn remove(mut self, k: K) -> ?V = todo

    /// Reserve space for at least `cap` keys.
    ///
    /// cost: O(cap) when the table grows, O(1) otherwise
    /// traps: never
    /// example: `var c = Counter.of([1]); c.reserve(100); c[1] == 1`
    pub fn reserve(mut self, cap: Int) = todo

    /// Remove all entries.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `var c = Counter.of([1, 2]); c.clear(); c.len == 0 and c[1] == 0`
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    ///
    /// The `init` of `self` is unchanged.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var c = Counter.of([1]); let d = c.take(); c.is_empty and d[1] == 1`
    pub fn take(mut self) -> Counter<K, V> = todo

    /// Merge all entries from `other` into `self` by adding values (emptying `other`).
    ///
    /// Stored values are added: a key stored in both ends at the sum of the two, a key stored in
    /// one keeps that value, and a sum equal to `init` is removed. Absent keys contribute
    /// nothing (not `init`), which keeps the result the same whichever side is drained.
    ///
    /// cost: O(min(|self|, |other|)) expected
    /// empty: an empty `other` leaves `self` unchanged
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `var a = Counter.of(['x', 'y']); var b = Counter.of(['y', 'z']); a.absorb(mut b); a['y'] == 2 and a.len == 3 and b.len == 0`
    pub fn absorb(mut self, mut other: Counter<K, V>) = todo

    /// Sum of all stored values.
    ///
    /// Only stored entries count; `init` is not added for absent keys.
    ///
    /// cost: O(n)
    /// empty: `V.zero`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `Counter.of([1, 1, 2]).total() == 3`
    pub fn total(self) -> V = todo

    /// Top `k` entries sorted by descending value.
    ///
    /// A `k` at least `len` returns every entry.
    ///
    /// cost: O(n log n)
    /// empty: `[]`
    /// ties: equal values in ascending key order
    /// order: descending value; a new `List`, keyed from 0
    /// traps: when `k < 0`
    /// example: `Counter.of([3, 1, 3, 2, 1, 3]).most_common(2) == [(3, 3), (1, 2)]`
    /// example: `Counter.of(['b', 'a']).most_common(5) == [('a', 1), ('b', 1)]`
    pub fn most_common(self, k: Int) -> List<(K, V)> = todo

    /// O(1) to make. Yields every stored `(key, val)` pair.
    ///
    /// cost: O(1) to make, O(1) per step
    /// empty: yields nothing
    /// order: unspecified hash order, the same as `keys` and `values` while unchanged
    /// traps: never
    /// example: `Counter.of([2, 1, 2]).items.sorted() == [(1, 1), (2, 2)]`
    @get pub fn items(self) -> yields (K, V) = todo

    ///
    ///
    /// cost: O(1) to make, O(1) per step
    /// empty: yields nothing
    /// order: unspecified hash order, the same as `items`
    /// traps: never
    /// example: `Counter.of([2, 1, 2]).keys.sorted() == [1, 2]`
    @get pub fn keys(self) -> yields K = todo

    ///
    ///
    /// cost: O(1) to make, O(1) per step
    /// empty: yields nothing
    /// order: unspecified hash order, the same as `items`
    /// traps: never
    /// example: `Counter.of([2, 1, 2]).values.sorted() == [1, 2]`
    @get pub fn values(self) -> yields V = todo

@test
fn flat_map_matches_builtin_hash_map():
    var fm = FlatMap<Int>(0)
    var ref = HashMap<Int, Int>()
    assert fm.is_empty and fm.len == 0 and fm.get(42) == None
    for _ in 0..300:
        let k = rand(-40, 40)
        let op = rand(0, 3)
        if op <= 1:
            let v = rand(-100, 100)
            fm.set(k, v)
            ref[k] = v
        elif op == 2:
            assert fm.remove(k) == ref.remove(k)
        else:
            assert fm.get(k) == ref.get(k)
            assert fm.contains(k) == (k in ref)
        assert fm.len == ref.len
    assert fm.get_or_insert(999, 7) == 7
    assert fm.get_or_insert(999, 8) == 7
    fm.clear()
    assert fm.is_empty and fm.get(999) == None

@test
fn hash_map_and_set_erase_clear_and_equality():
    var hm = HashMap<Int, Int>()
    var ord = Map<Int, Int>()
    var hs = Set<Int>()
    var os = Set<Int>()
    for _ in 0..400:
        let k = rand(-30, 30) * 64
        let op = rand(0, 3)
        if op <= 1:
            let v = rand(-100, 100)
            hm[k] = v
            ord[k] = v
            assert hs.insert(k) == os.insert(k)
        elif op == 2:
            assert hm.remove(k) == ord.remove(k)
            assert hs.remove(k) == os.remove(k)
        else:
            assert hm.get(k) == ord.get(k)
            assert (k in hs) == (k in os)
        assert hm.len == ord.len and hs.len == os.len
    var hm2 = HashMap<Int, Int>()
    for (k, v) in ord: hm2[k] = v
    assert hm == hm2
    for x in os: assert x in hs
    var bs = Set<Bool>()
    assert bs.insert(true) and not bs.insert(true) and (true in bs) and not (false in bs)
    assert bs.remove(true) and bs.is_empty
    hm.clear()
    hs.clear()
    assert hm.is_empty and hs.is_empty
