//! Fixed-domain array with O(1) point updates and O(changes) snapshot/rollback (`RollbackArray<D, T>`).

/// A `[D]T` array that records every write on an undo log so offline divide-and-conquer
/// over time (e.g. segment tree over queries) can snapshot and roll back in O(1) amortized.
pub struct RollbackArray<D, T>:
    data: [D]T
    log: List<(D, T)>

    pub fn new(init: [D]T) -> RollbackArray<D, T> = RollbackArray(init, [])

    pub fn filled(val: T) -> RollbackArray<D, T> = RollbackArray([D] _ => val, [])

    /// Read `data[i]`. O(1).
    pub fn get(self, i: D) -> T = data[i]

    /// Write `data[i] = val`, recording the previous value on the undo log. O(1).
    pub fn set(mut self, i: D, val: T):
        log.push((i, data[i]))
        data[i] = val

    /// Current checkpoint for a later `rollback`. O(1).
    pub fn save(self) -> Int = log.len

    /// Undo every `set` performed since checkpoint `t`. O(changes).
    pub fn rollback(mut self, t: Int):
        assert 0 <= t and t <= log.len
        while log.len > t:
            let (i, old) = log.pop()!
            data[i] = old

@test
fn rollback_array_matches_snapshot_copies():
    domain D = 0..8
    var arr = RollbackArray<D, Int>.filled(0)
    var snaps: List<(Int, List<Int>)> = []
    var model = [0 for _ in 0..8]
    for _ in 0..80:
        let op = rand(0, 3)
        if op == 0:
            snaps.push((arr.save(), model.copy()))
        elif op == 1 and not snaps.is_empty:
            let (t, saved) = snaps.pop()!
            arr.rollback(t)
            model = saved
        else:
            let i = rand(0, 7)
            let v = rand(-50, 50)
            arr.set(D(i), v)
            model[i] = v
        assert [arr[i] for i in D] == model
