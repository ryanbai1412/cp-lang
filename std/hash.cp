//! Hashing and the hash tables built on it: `Hash`, `HashMap` and `HashSet`. A table finds a key
//! by its salted `hash` and compares keys with `==`, in \(O(1)\) expected per lookup, insert and
//! remove. Walking one has no defined order, so neither is `Show` (R10.3.8); `Map` and `Set` are
//! the ordered spelling (lib/containers.md §2.8).
//!
//! start: `HashMap`, `HashSet`, `Hash`
//! example: `var h = HashMap<Int, Int>(); h[1] = 10; h.get(1) == Some(10) and h.get(2) == None`

/// A value's 64-bit hash: equal values hash equal. `Int`, `I128`, `U64`, `Char`, `Bool` and
/// `Str` have it here (splitmix64, decision 28.8); a struct gets it from `hash_fields`, and
/// `@structural` gives a tuple, record, optional or array it when each part has it.
@structural
pub trait Hash:
    fn hash(self) -> U64

/// SplitMix64: mixes the 64 bits of `x` so that inputs close together land far apart.
///
/// One full splitmix64 step on state `x`: `z = x + 0x9E3779B97F4A7C15`, then `z ^= z >> 30; z *=
/// 0xBF58476D1CE4E5B9; z ^= z >> 27; z *= 0x94D049BB133111EB; z ^= z >> 31`, all wrapping. It is
/// the usual guard for `HashMap` keys, and `std.rand.Rng.next` is this applied to its state.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `splitmix64(0) == 0xE220A8397B1DCDAF`
/// example: `splitmix64(1) != splitmix64(2)`
pub fn splitmix64(x: U64) -> U64:
    var z: U64 = x + 0x9e3779b97f4a7c15
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9
    z = (z ^ (z >> 27)) * 0x94d049bb133111eb
    z ^ (z >> 31)

impl Hash for Int:
    /// cost: \(O(1)\)
    /// example: `5.hash() == 5.hash()`
    fn hash(self) -> U64 = splitmix64(U64(self))

impl Hash for I128:
    /// cost: \(O(1)\)
    /// example: `I128(-5).hash() != I128(5).hash()`
    fn hash(self) -> U64:
        let low = (I128(1) << 64) - I128(1)
        splitmix64(U64(self & low) ^ splitmix64(U64((self >> 64) & low)))

impl Hash for U64:
    /// cost: \(O(1)\)
    fn hash(self) -> U64 = splitmix64(self)

impl Hash for Char:
    /// cost: \(O(1)\)
    fn hash(self) -> U64 = splitmix64(U64(Int(self)))

impl Hash for Bool:
    /// cost: \(O(1)\)
    fn hash(self) -> U64 = splitmix64(if self: U64(1) else: U64(0))

impl Hash for Str:
    /// cost: \(O(\mathit{length})\)
    /// example: `"ab".hash() != "ba".hash()`
    fn hash(self) -> U64:
        var h = U64(self.len)
        for c in self: h = splitmix64(h ^ U64(Int(c)))
        h


impl<A: Hash> Hash for Tuple1<A>:
    fn hash(self) -> U64 = splitmix64(self.0.hash())

impl<A: Hash, B: Hash> Hash for Tuple2<A, B>:
    fn hash(self) -> U64 = splitmix64(splitmix64(self.0.hash()) ^ self.1.hash())

impl<A: Hash, B: Hash, C: Hash> Hash for Tuple3<A, B, C>:
    fn hash(self) -> U64 = splitmix64(splitmix64(splitmix64(self.0.hash()) ^ self.1.hash()) ^ self.2.hash())

impl<A: Hash, B: Hash, C: Hash, E: Hash> Hash for Tuple4<A, B, C, E>:
    fn hash(self) -> U64 = splitmix64(splitmix64(splitmix64(splitmix64(self.0.hash()) ^ self.1.hash()) ^ self.2.hash()) ^ self.3.hash())

fn home(h: U64, mask: Int) -> Int = Int(splitmix64(h ^ U64(0x5bd1e995)) & U64(mask))

/// A hash table from `K` to `V` whatever the program observes: C++ `unordered_map`, Python
/// `dict`, Rust `HashMap`. Entries sit densely in insertion slots and an open-addressing table
/// (linear probing, load at most 1/2, backward-shift removal) points at them. `m[k]` traps when
/// `k` is absent (`m.get(k)` is the `?V`), `m[k] = v` inserts or replaces, and `m[k] op= v`
/// starts an absent key at `V.default()`.
pub struct HashMap<K, V>:
    var ks: List<K> = []
    var vs: List<V> = []
    var table: List<?Int> = []

    /// An empty map; `HashMap(pairs)` builds one from `(key, value)` pairs.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `HashMap<Int, Int>().len == 0`
    pub fn new() -> HashMap<K, V> = HashMap([], [], [])

impl<K: Hash + Eq, V> HashMap<K, V>:
    /// The number of keys.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[3] = 1; h[3] = 2; h.len == 1`
    @get pub fn len(self) -> Int = ks.len

    /// Whether no key is present.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `HashMap<Int, Int>().is_empty`
    @get pub fn is_empty(self) -> Bool = ks.len == 0

    /// Remove every key.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[1] = 1; h.clear(); h.is_empty`
    pub fn clear(mut self):
        ks = []
        vs = []
        table = []

    /// Make room for at least `cap` keys without the table growing again.
    ///
    /// cost: \(O(\mathit{cap})\) when the table grows, \(O(1)\) otherwise
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h.reserve(100); h[1] = 1; h.len == 1`
    pub fn reserve(mut self, cap: Int):
        var size = if table.len == 0: 8 else: table.len
        while size < 2 * cap: size *= 2
        if size == table.len: return
        table = List.filled(size, None)
        for d in 0..ks.len: table[self.slot(ks[d])] = Some(d)

    fn slot(self, k: K) -> Int:
        let mask = table.len - 1
        var i = home(k.hash(), mask)
        while table[i] != None and ks[table[i]!] != k: i = (i + 1) & mask
        i

    fn find(self, k: K) -> ?Int = if table.len == 0: None else: table[self.slot(k)]

    fn grow(mut self):
        table = List.filled(if table.len == 0: 8 else: 2 * table.len, None)
        for d in 0..ks.len: table[self.slot(ks[d])] = Some(d)

    fn put(mut self, k: K, v: V) -> Int:
        if 2 * (ks.len + 1) > table.len: self.grow()
        let i = self.slot(k)
        if let d = table[i]:
            vs[d] = v
            return d
        table[i] = Some(ks.len)
        ks.push(k)
        vs.push(v)
        ks.len - 1

    /// The value at `k`, or None when `k` is absent.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[1] = 10; h.get(1) == Some(10) and h.get(2) == None`
    pub fn get(self, k: K) -> ?V:
        let d = self.find(k)
        if d == None: return None
        Some(vs[d!])

    /// Remove `k`, giving its value, or None when it was absent.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[1] = 10; h.remove(1) == Some(10) and h.remove(1) == None`
    pub fn remove(mut self, k: K) -> ?V:
        if table.len == 0: return None
        var i = self.slot(k)
        let found = table[i]
        if found == None: return None
        let d = found!
        let out = vs[d]
        let mask = table.len - 1
        var j = (i + 1) & mask
        while table[j] != None:
            let h = home(ks[table[j]!].hash(), mask)
            if (i < j and (h <= i or h > j)) or (i > j and h <= i and h > j):
                table[i] = table[j]
                i = j
            j = (j + 1) & mask
        let empty: ?Int = None
        table[i] = empty
        let last = ks.len - 1
        if d != last:
            table[self.slot(ks[last])] = Some(d)
            ks.swap(d, last)
            vs.swap(d, last)
        let _ = ks.pop()
        let _ = vs.pop()
        Some(out)

    /// Each key with its value, in no defined order.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[1] = 10; h[2] = 20; h.items.collect().sorted() == [(1, 10), (2, 20)]`
    @get pub fn items(self) -> yields (K, V):
        for d in 0..ks.len: yield (ks[d], vs[d])

    /// Each key, in no defined order.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[2] = 0; h[1] = 0; h.keys.collect().sorted() == [1, 2]`
    @get pub fn keys(self) -> yields K:
        for k in ks: yield k

    /// Each value, in no defined order.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `var h = HashMap<Int, Int>(); h[1] = 5; h.values.collect() == [5]`
    @get pub fn values(self) -> yields V:
        for v in vs: yield v

impl<K: Hash + Eq, V> Contains<K> for HashMap<K, V>:
    fn contains(self, k: K) -> Bool = self.find(k) != None

impl<K: Hash + Eq, V: Default> IndexMut<K, V> for HashMap<K, V>:
    fn at(mut self, k: K) -> lends mut V:
        let d = self.find(k) ?? self.put(k, V.default())
        lend mut self.vs[d]

impl<K: Hash + Eq, V> IndexSet<K, V> for HashMap<K, V>:
    fn set(mut self, k: K, v: V):
        let _ = self.put(k, v)

impl<K: Hash + Eq, V: Eq> Eq for HashMap<K, V>:
    /// The same keys, each with an equal value, whatever order they went in.
    fn eq(self, other: HashMap<K, V>) -> Bool:
        if ks.len != other.len: return false
        for d in 0..ks.len:
            if other.get(ks[d]) != Some(vs[d]): return false
        true

/// `HashMap(pairs)`: each `(key, value)` in order, a later pair replacing an earlier one with an
/// equal key, as Python's `dict(pairs)` builds.
///
/// cost: \(O(n)\) expected
/// example: `HashMap([(1, 2), (3, 4), (1, 5)])[1] == 5`
impl<K: Hash + Eq, V> Into<HashMap<K, V>> for List<(K, V)>:
    fn into(self) -> HashMap<K, V>:
        var m = HashMap<K, V>()
        m.reserve(self.len)
        for (k, v) in self: let _ = m.put(k, v)
        m

/// A hash set of `T` whatever the program observes: C++ `unordered_set`, Python `set`, Rust
/// `HashSet`. It is a `HashMap<T, ()>`.
pub struct HashSet<T>:
    var m: HashMap<T, ()> = HashMap()

    /// An empty set; `HashSet(xs)` builds one from the elements of `xs`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `HashSet<Int>().len == 0`
    pub fn new() -> HashSet<T> = HashSet(HashMap<T, ()>())

impl<T: Hash + Eq> HashSet<T>:
    /// The number of elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = HashSet<Int>(); s.insert(3); s.insert(3); s.len == 1`
    @get pub fn len(self) -> Int = m.len

    /// Whether no element is present.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `HashSet<Int>().is_empty`
    @get pub fn is_empty(self) -> Bool = m.len == 0

    /// Remove every element.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var s = HashSet<Int>(); s.insert(1); s.clear(); s.is_empty`
    pub fn clear(mut self):
        m.clear()

    /// Add `x`; true when it was absent.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var s = HashSet<Int>(); s.insert(3) and not s.insert(3)`
    pub fn insert(mut self, x: T) -> Bool:
        let n = m.len
        let _ = m.put(x, ())
        m.len > n

    /// `insert` by its other name: add `x`; true when it was absent.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var s = HashSet<Int>(); s.add(3) and not s.add(3)`
    pub fn add(mut self, x: T) -> Bool = self.insert(x)

    /// Remove `x`; true when it was present.
    ///
    /// cost: \(O(1)\) expected
    /// traps: never
    /// example: `var s = HashSet<Int>(); s.insert(3); s.remove(3) and not s.remove(3)`
    pub fn remove(mut self, x: T) -> Bool = m.remove(x) != None

    /// Each element, in no defined order.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `var s = HashSet<Int>(); s.insert(2); s.insert(1); s.items.collect().sorted() == [1, 2]`
    @get pub fn items(self) -> yields T:
        for x in m.ks: yield x

impl<T: Hash + Eq> Contains<T> for HashSet<T>:
    fn contains(self, x: T) -> Bool = m.find(x) != None

impl<T: Hash + Eq> Eq for HashSet<T>:
    /// The same elements, whatever order they went in.
    fn eq(self, other: HashSet<T>) -> Bool:
        if m.len != other.len: return false
        for x in m.ks:
            if not (x in other): return false
        true

/// `HashSet(xs)`: the distinct elements of `xs`, as Python's `set(xs)` builds.
///
/// cost: \(O(n)\) expected
/// example: `HashSet([3, 1, 3]).len == 2`
impl<T: Hash + Eq> Into<HashSet<T>> for List<T>:
    fn into(self) -> HashSet<T>:
        var s = HashSet<T>()
        s.m.reserve(self.len)
        for x in self: let _ = s.m.put(x, ())
        s

@test
fn splitmix64_matches_the_reference_values():
    assert splitmix64(0) == 0xe220a8397b1dcdaf
    assert splitmix64(0x9e3779b97f4a7c15) == 0x6e789e6aa1b965f4
    assert splitmix64(1) != splitmix64(2)
