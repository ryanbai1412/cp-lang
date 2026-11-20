//! Monotone queue: sliding-window minimum or maximum in O(1) amortized per element.

/// A monotone queue: the minimum (`MonoDeque<Int>(min)`) or maximum (`MonoDeque<Int>(max)`)
/// of a window that only moves right. `push(at, val)` enters the value at position `at`,
/// `drop_before(min_at)` forgets positions left of `min_at`, `best` is the answer for what is
/// left. Positions must be pushed in increasing order. Amortized O(1) each.
pub struct MonoDeque<T>(keep: (T, T) -> Bool):
    q: List<(at: Int, val: T)>
    head: Int

    pub fn new(pick: (T, T) -> T) -> MonoDeque<T>:
        // `pick` is `min` or `max`: a value stays while it is strictly better than a newcomer
        MonoDeque((x, y) => pick(x, y) == x and x != y, [], 0)

    /// A queue ordered by your own "strictly better than".
    pub fn by(better: (T, T) -> Bool) -> MonoDeque<T> = MonoDeque(better, [], 0)

    pub @get fn len(self) -> Int = q.len - head
    pub @get fn is_empty(self) -> Bool = q.len == head

    pub fn clear(mut self):
        q = []
        head = 0

    /// Enter `val` at position `at`; values it beats or ties leave, since they can never win again.
    pub fn push(mut self, at: Int, val: T):
        while q.len > head and not keep(q[q.len - 1].val, val): q.pop()
        q.push((at: at, val: val))

    /// Forget every position left of `min_at`.
    pub fn drop_before(mut self, min_at: Int):
        while head < q.len and q[head].at < min_at: head += 1
        if head > 32 and 2 * head > q.len:
            q = [q[i] for i in head..q.len]
            head = 0

    /// The best value still in the window, or `none` when it is empty.
    pub @get fn best(self) -> ?T = if q.len == head: none else: q[head].val

@test
fn a_monotone_queue_by_hand():
    // the best value among the last 3 positions, skipping walls
    let hall = "ab.xd.c"
    var q = MonoDeque<Char>(max)
    var seen: List<?Char> = []
    for i in 0..hall.len:
        if hall[i] == 'x':
            q.clear()
            seen.push(none)
            continue
        if hall[i] != '.': q.push(i, hall[i])
        q.drop_before(i - 2)
        seen.push(q.best)
    assert seen == ['a', 'b', 'b', none, 'd', 'd', 'd']
    var low = MonoDeque<Int>(min)
    for (i, x) in [(0, 5), (1, 3), (2, 3), (3, 8)]: low.push(i, x)
    assert low.best == 3 and low.len == 2
    low.drop_before(3)
    assert low.best == 8 and not low.is_empty

@test
fn sliding_window_min_and_max_match_brute_force():
    for _ in 0..50:
        let a = [rand(-9, 9) for _ in 0..rand(1, 14)]
        let k = rand(1, a.len)
        var qmin = MonoDeque<Int>(min)
        var qmax = MonoDeque<Int>(max)
        var mins: List<Int> = []
        var maxs: List<Int> = []
        for i in 0..a.len:
            qmin.push(i, a[i])
            qmax.push(i, a[i])
            qmin.drop_before(i - k + 1)
            qmax.drop_before(i - k + 1)
            if i >= k - 1:
                mins.push(qmin.best!)
                maxs.push(qmax.best!)
        assert mins == [min(a[j] for j in i..i + k)! for i in 0..=a.len - k]
        assert maxs == [max(a[j] for j in i..i + k)! for i in 0..=a.len - k]
