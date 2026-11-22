//! Bidirectional key <-> integer id interner (`Interner<K>`).

/// Maps arbitrary keys `K` (e.g. `Str`, coordinates, pairs) to dense 0-based ids `0..len`
/// on the fly, supporting both forward lookup `.id(k)` and reverse lookup `.key(i)`.
pub struct Interner<K>:
    to_id: Map<K, Int>
    to_key: List<K>

    pub fn new() -> Interner<K> = Interner(Map<K, Int>(), [])

    /// Number of interned keys.
    pub @get fn len(self) -> Int = to_key.len

    pub @get fn is_empty(self) -> Bool = to_key.is_empty

    /// Return the id for `k`, assigning the next id if `k` has not been seen yet. O(log n).
    pub fn id(mut self, k: K) -> Int:
        if k in to_id: return to_id[k]
        let idx = to_key.len
        to_id[k] = idx
        to_key.push(k)
        idx

    /// Return the existing id for `k`, or `none` without inserting. O(log n).
    pub fn get(self, k: K) -> ?Int:
        if k in to_id: to_id[k] else: none

    /// Return the key assigned to `idx`. O(1).
    pub fn key(self, idx: Int) -> K = to_key[idx]

    /// All interned keys in id order.
    pub @get fn keys(self) -> List<K> = to_key

@test
fn interner_assigns_dense_ids_and_round_trips():
    var ids = Interner<Str>()
    assert ids.is_empty and ids.get("alice") == none
    let a = ids.id("alice")
    let b = ids.id("bob")
    let a2 = ids.id("alice")
    let c = ids.id("carol")
    assert a == 0 and b == 1 and a2 == 0 and c == 2
    assert ids.len == 3 and ids.get("bob") == 1 and ids.get("dave") == none
    assert ids.key(0) == "alice" and ids.key(1) == "bob" and ids.key(2) == "carol"
    assert ids.keys == ["alice", "bob", "carol"]
