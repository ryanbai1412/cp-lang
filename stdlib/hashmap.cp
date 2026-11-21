//! Open-addressing hash map for `Int` keys with splitmix64 salting.
//!
//! `FlatMap<V>` stores keys, values, and occupancy states in flat parallel arrays with
//! linear probing and power-of-two capacity, avoiding per-node heap allocations when
//! built-in `HashMap<Int, V>` overhead matters.
use std.math.{splitmix64}

// Slot states: 0 = empty, 1 = occupied, 2 = tombstone.
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

    /// Create an empty `FlatMap<V>` with initial capacity 16. O(1).
    pub fn new(default_val: V) -> FlatMap<V> = FlatMap<V>.with_capacity(16, default_val)

    /// Create an empty `FlatMap<V>` sized for at least `cap` entries. O(cap).
    pub fn with_capacity(cap: Int, default_val: V) -> FlatMap<V>:
        var sz = 16
        while sz < cap * 2: sz *= 2
        let s = splitmix64(U64(0x517cc1b727220a95))
        FlatMap(
            List.filled(sz, 0),
            List.filled(sz, default_val),
            List.filled(sz, 0),
            default_val,
            s,
            0,
            0,
        )

    /// Number of live key-value pairs. O(1).
    pub @get fn len(self) -> Int = live

    /// Whether the map has no live entries. O(1).
    pub @get fn is_empty(self) -> Bool = live == 0

    /// Remove all entries while keeping allocated capacity. O(capacity).
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

    /// Look up `k`, returning `none` if absent. O(1) expected.
    pub fn get(self, k: Int) -> ?V:
        let mask = state.len - 1
        var i = self.slot(k)
        while state[i] != 0:
            if state[i] == 1 and keys[i] == k: return vals[i]
            i = (i + 1) & mask
        none

    /// Whether `k` is currently present. O(1) expected.
    pub fn contains(self, k: Int) -> Bool = self.get(k) != none

    /// Insert or overwrite `k -> v`. O(1) expected amortized.
    pub fn set(mut self, k: Int, v: V):
        if (used + 1) * 10 >= state.len * 7:
            let next_cap = if live * 4 < state.len: state.len else: state.len * 2
            self.rehash(next_cap)
        let mask = state.len - 1
        var i = self.slot(k)
        var tomb = -1
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

    /// Return the value for `k`, inserting `v` first if `k` is absent. O(1) expected amortized.
    pub fn get_or_insert(mut self, k: Int, v: V) -> V:
        if let cur = self.get(k): return cur
        self.set(k, v)
        v

    /// Remove `k` if present; returns its old value or `none`. O(1) expected.
    pub fn remove(mut self, k: Int) -> ?V:
        let mask = state.len - 1
        var i = self.slot(k)
        while state[i] != 0:
            if state[i] == 1 and keys[i] == k:
                state[i] = 2
                live -= 1
                return vals[i]
            i = (i + 1) & mask
        none

@test
fn flat_map_matches_builtin_hash_map():
    var fm = FlatMap<Int>(0)
    var ref = HashMap<Int, Int>()
    assert fm.is_empty and fm.len == 0 and fm.get(42) == none
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
    assert fm.is_empty and fm.get(999) == none

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
