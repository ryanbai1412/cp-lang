//! Double-ended queue: `Deque<T>`.

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

    /// Move the deque's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> Deque<T>:
        let out = Deque(buf, head, n)
        buf = [none]
        head = 0
        n = 0
        out

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
    for x in [10, 20, 30]: q.push(x)
    var moved = q.take()
    assert q.is_empty and moved.len == 3 and moved.pop_front() == 10 and moved.pop() == 30

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

