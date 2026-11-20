//! Generation-stamped mark table: `add`, `in`, `remove`, and `clear` are all O(1).
//!
//! Use `Marks<D>()` whenever clearing a `[D]Bool` inside a loop would cost O(T * N).
//! Each `clear()` bumps a generation counter instead of touching the underlying array.

/// A visited/mark set over a domain `D` with O(1) `add`, `remove`, `contains` (`v in m`),
/// and O(1) `clear()`. Memory is one `[D]Int` array of generation stamps.
pub struct Marks<D>:
    stamp: [D]Int
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

    /// Clear every mark in O(1) without touching the array.
    pub fn clear(mut self):
        gen += 1
        cnt = 0

    /// How many positions are marked in the current round. O(1).
    pub @get fn len(self) -> Int = cnt

    pub @get fn is_empty(self) -> Bool = cnt == 0

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
