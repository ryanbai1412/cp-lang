//! Double-ended queue: `Deque<T>`.

use std.seq.{remove}

/// A queue with both ends open: `var q: Deque<Int> = []`. `push` / `pop` work at the back,
/// `push_front` / `pop_front` at the front, all amortized O(1); `q[i]` is the `i`-th element
/// from the front in O(1). Memory is a ring of at most twice the largest length.
pub struct Deque<T>:
    buf: List<T>
    head: Int
    n: Int
    mask: Int
    want: Int

    /// Create an empty `Deque<T>`. O(1).
    pub fn new() -> Deque<T> = Deque([], 0, 0, -1, 1)

    /// A deque holding `xs`, front to back: the queue of a search, `var q = Deque.of(sources)`,
    /// then `while let u = q.pop_front(): ...`. O(n).
    pub fn of(xs: List<T>) -> Deque<T>:
        var q = Deque<T>.new()
        q.reserve(xs.len)
        for x in xs: q.push(x)
        q

    /// Reserve capacity for at least `cap` elements without reallocating. O(1) when empty, O(cap) otherwise.
    pub fn reserve(mut self, cap: Int):
        var c = 1
        while c < cap: c *= 2
        if c <= mask + 1: return
        if buf.is_empty:
            if c > want: want = c
        else:
            self.grow_to(c, buf[head])

    /// Number of elements currently in the deque. O(1).
    pub @get fn len(self) -> Int = n
    /// Whether the deque is empty. O(1).
    pub @get fn is_empty(self) -> Bool = n == 0

    /// Remove all elements from the deque. O(1).
    pub fn clear(mut self):
        buf = []
        head = 0
        n = 0
        mask = -1
        want = 1

    /// Move the deque's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> Deque<T>:
        let out = Deque(buf, head, n, mask, want)
        buf = []
        head = 0
        n = 0
        mask = -1
        want = 1
        out

    fn grow_to(mut self, cap: Int, fill: T):
        let m = mask
        var nb = List.filled(cap, fill)
        unchecked:
            for i in 0..n:
                nb[i] = buf[(head + i) & m]
        swap(mut buf, mut nb)
        head = 0
        mask = cap - 1

    /// Append `x` to the back of the deque. O(1) amortized.
    pub fn push(mut self, x: T):
        if n > mask:
            let cap = if mask < 0: want else: 2 * (mask + 1)
            self.grow_to(cap, x)
        unchecked:
            buf[(head + n) & mask] = x
        n += 1

    /// Prepend `x` to the front of the deque. O(1) amortized.
    pub fn push_front(mut self, x: T):
        if n > mask:
            let cap = if mask < 0: want else: 2 * (mask + 1)
            self.grow_to(cap, x)
        unchecked:
            head = (head - 1) & mask
            buf[head] = x
        n += 1

    /// Remove and return the back, or `none` when empty.
    pub fn pop(mut self) -> ?T:
        if n == 0: return none
        n -= 1
        unchecked:
            return some(buf[(head + n) & mask])

    /// Remove and return the front, or `none` when empty.
    pub fn pop_front(mut self) -> ?T:
        if n == 0: return none
        unchecked:
            let x = buf[head]
            head = (head + 1) & mask
            n -= 1
            return some(x)

    /// The front, or `none` when empty.
    pub @get fn first(self) -> ?T:
        if n == 0: return none
        unchecked:
            return some(buf[head])

    /// The back, or `none` when empty. `dq.last(k)` is the element `k`
    /// back from it, so `dq.last(1)` is the one before the back.
    pub @get fn last(self, k: Int = 0) -> ?T:
        if k < 0 or k >= n: return none
        unchecked:
            return some(buf[(head + n - 1 - k) & mask])

    /// Each element with its place from the front: `for (i, x) in dq.items`. O(n).
    pub @get fn items(self) -> List<(Int, T)> = [(i, buf[(head + i) & mask]) for i in 0..n]

    /// The `i`-th element from the front.
    pub fn get(self, i: Int) -> T:
        assert 0 <= i and i < n
        unchecked:
            return buf[(head + i) & mask]

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
            if not xs.is_empty: xs.remove(0)
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

@test
fn a_deque_of_a_list_is_a_bfs_queue():
    // BFS on a line 0 - 1 - 2 - 3 - 4 from both ends
    domain V = 0..5
    var dist: [V]?Int = none
    let ends = [V.first, V.last]
    for v in ends: dist[v] = 0
    var q = Deque.of(ends)
    assert q.len == 2 and q.first == V.first and q.last == V.last
    while let u = q.pop_front():
        for d in [-1, 1]:
            if let w = u + d:
                if dist[w] == none:
                    dist[w] = dist[u]! + 1
                    q.push(w)
    assert [dist[v]! for v in V] == [0, 1, 2, 1, 0]
    assert Deque<Int>.of([]).is_empty

@test
fn a_deque_reserve_preserves_order():
    var q: Deque<Int> = []
    q.reserve(64)
    for i in 0..20: q.push(i)
    for i in 1..=10: q.push_front(-i)
    q.reserve(256)
    q.reserve(4)
    assert q.len == 30
    assert q[0] == -10 and q[10] == 0 and q[29] == 19
    assert q.pop_front() == -10 and q.pop() == 19
