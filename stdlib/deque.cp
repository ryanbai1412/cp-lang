//! Double-ended queues: `Deque<T>` and the monotone `MonoDeque<T>`.

/// A queue with both ends open: `var q: Deque<Int> = []`. `push` / `pop` work at the back,
/// `push_front` / `pop_front` at the front, all amortized O(1); `q[i]` is the `i`-th element
/// from the front in O(1). Memory is a ring of at most twice the largest length.
pub struct Deque<T>:
    buf: List<?T>
    head: Int
    n: Int

    pub fn new() -> Deque<T> = Deque([none], 0, 0)

    pub @get fn len(self) -> Int = n
    pub @get fn is_empty(self) -> Bool = n == 0

    pub fn clear(mut self):
        buf = [none]
        head = 0
        n = 0

    fn grow(mut self):
        if n < buf.len: return
        let old = [buf[(head + i) % buf.len] for i in 0..n]
        buf = List.filled(2 * old.len, none)
        for i in 0..n: buf[i] = old[i]
        head = 0

    pub fn push(mut self, x: T):
        self.grow()
        buf[(head + n) % buf.len] = x
        n += 1

    pub fn push_front(mut self, x: T):
        self.grow()
        head = (head - 1) % buf.len
        buf[head] = x
        n += 1

    /// Remove and return the back, or `none` when empty.
    pub fn pop(mut self) -> ?T:
        if n == 0: return none
        n -= 1
        buf[(head + n) % buf.len]

    /// Remove and return the front, or `none` when empty.
    pub fn pop_front(mut self) -> ?T:
        if n == 0: return none
        let x = buf[head]
        head = (head + 1) % buf.len
        n -= 1
        x

    /// The front, or `none` when empty.
    pub @get fn first(self) -> ?T = if n == 0: none else: buf[head]

    /// The back, or `none` when empty.
    pub @get fn last(self) -> ?T = if n == 0: none else: buf[(head + n - 1) % buf.len]

    /// The `i`-th element from the front.
    pub fn get(self, i: Int) -> T:
        assert 0 <= i and i < n
        buf[(head + i) % buf.len]!

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
fn a_deque_matches_a_list():
    var q: Deque<Int> = []
    var xs: List<Int> = []
    for _ in 0..400:
        let k = rand(0, 5)
        if k == 0:
            let x = rand(0, 99)
            q.push(x)
            xs.push(x)
        elif k == 1:
            let x = rand(0, 99)
            q.push_front(x)
            xs = [x] + xs
        elif k == 2:
            assert q.pop() == (if xs.is_empty: none else: xs.pop())
        elif k == 3:
            let want = xs.first
            if not xs.is_empty: xs = [xs[i] for i in 1..xs.len]
            assert q.pop_front() == want
        else:
            assert q.len == xs.len and q.is_empty == xs.is_empty
            assert q.first == xs.first and q.last == xs.last
            for i in 0..xs.len: assert q[i] == xs[i]
    q.clear()
    assert q.is_empty and q.pop() == none and q.pop_front() == none

@test
fn a_deque_as_a_bfs_queue():
    // 0-1 BFS on a line: stepping right is free, stepping left costs 1
    var dist: List<?Int> = [none, none, none, none, none]
    var q = Deque<Int>()
    dist[2] = 0
    q.push(2)
    while let u = q.pop_front():
        if u + 1 < 5 and (dist[u + 1] == none or dist[u + 1]! > dist[u]!):
            dist[u + 1] = dist[u]
            q.push_front(u + 1)
        if u >= 1 and (dist[u - 1] == none or dist[u - 1]! > dist[u]! + 1):
            dist[u - 1] = dist[u]! + 1
            q.push(u - 1)
    assert dist == [2, 1, 0, 0, 0]

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

