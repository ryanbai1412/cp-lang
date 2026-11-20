//! A sequence that can be cut and glued anywhere: insert, erase, reverse a stretch and
//! read a position, each in O(log n) expected. An implicit treap over flat arrays.

/// A sequence of `T`. Positions are plain `Int`s counted from 0, because they shift with
/// every edit; a `Range<Int>` names a stretch.
pub struct Rope<T>:
    val: List<T>
    pri: List<Int>
    left: List<?Int>
    right: List<?Int>
    size: List<Int>
    /// Is the subtree waiting to be mirrored?
    flip: List<Bool>
    root: ?Int

    pub fn new() -> Rope<T> = Rope([], [], [], [], [], [], none)

    /// The rope holding `xs` in order.
    pub fn of(xs: List<T>) -> Rope<T>:
        var r = Rope<T>.new()
        for x in xs: r.push(x)
        r

    pub @get fn len(self) -> Int = self.count(root)

    /// Append `x` at the end.
    pub fn push(mut self, x: T):
        let node = self.node(x)
        root = self.merge(root, node)

    /// Insert `x` so that it ends up at position `i` (`0..=len`).
    pub fn insert(mut self, i: Int, x: T):
        assert 0 <= i <= self.len
        let (a, b) = self.split(root, i)
        let node = self.node(x)
        let ab = self.merge(a, node)
        root = self.merge(ab, b)

    /// Remove and return the value at position `i`.
    pub fn remove(mut self, i: Int) -> T:
        assert 0 <= i < self.len
        let (a, rest) = self.split(root, i)
        let (mid, b) = self.split(rest, 1)
        root = self.merge(a, b)
        val[mid!]

    /// Reverse the stretch `r` in place.
    pub fn reverse(mut self, r: Range<Int>):
        let (a, rest) = self.split(root, r.lo)
        let (mid, b) = self.split(rest, r.len)
        if mid != none: flip[mid!] = not flip[mid!]
        let am = self.merge(a, mid)
        root = self.merge(am, b)

    /// Cut the stretch `r` out and glue it back in so that it starts at position `to` of
    /// what is left.
    pub fn move_to(mut self, r: Range<Int>, to: Int):
        let (a, rest) = self.split(root, r.lo)
        let (mid, b) = self.split(rest, r.len)
        let others = self.merge(a, b)
        let (x, y) = self.split(others, to)
        let xm = self.merge(x, mid)
        root = self.merge(xm, y)

    /// The value at position `i`.
    pub fn get(mut self, i: Int) -> T:
        assert 0 <= i < self.len
        var (at, k) = (root!, i)
        while true:
            self.push_down(at)
            let before = self.count(left[at])
            if k < before:
                at = left[at]!
            elif k == before:
                return val[at]
            else:
                k -= before + 1
                at = right[at]!

    /// Every value, in order. O(n).
    pub fn to_list(mut self) -> List<T>:
        var out: List<T> = []
        var stack: List<Int> = []
        var at = root
        while at != none or not stack.is_empty:
            while at != none:
                self.push_down(at!)
                stack.push(at!)
                at = left[at!]
            let v = stack.pop()!
            out.push(val[v])
            at = right[v]
        out

    fn count(self, t: ?Int) -> Int = if t == none: 0 else: size[t!]

    fn node(mut self, x: T) -> Int:
        val.push(x)
        pri.push(rand(0, 1 << 60))
        left.push(none)
        right.push(none)
        size.push(1)
        flip.push(false)
        val.len - 1

    fn push_down(mut self, t: Int):
        if not flip[t]: return
        let (l, r) = (left[t], right[t])
        left[t] = r
        right[t] = l
        if l != none: flip[l!] = not flip[l!]
        if r != none: flip[r!] = not flip[r!]
        flip[t] = false

    fn pull(mut self, t: Int):
        size[t] = 1 + self.count(left[t]) + self.count(right[t])

    /// The first `k` values of `t`, and the rest.
    fn split(mut self, t: ?Int, k: Int) -> (?Int, ?Int):
        if t == none: return (none, none)
        let v = t!
        self.push_down(v)
        if self.count(left[v]) >= k:
            let (a, b) = self.split(left[v], k)
            left[v] = b
            self.pull(v)
            (a, v)
        else:
            let (a, b) = self.split(right[v], k - self.count(left[v]) - 1)
            right[v] = a
            self.pull(v)
            (v, b)

    fn merge(mut self, a: ?Int, b: ?Int) -> ?Int:
        if a == none: return b
        if b == none: return a
        let (x, y) = (a!, b!)
        if pri[x] >= pri[y]:
            self.push_down(x)
            let m = self.merge(right[x], y)
            right[x] = m
            self.pull(x)
            x
        else:
            self.push_down(y)
            let m = self.merge(x, left[y])
            left[y] = m
            self.pull(y)
            y

@test
fn edits_match_a_plain_list():
    for _ in 0..12:
        var r = Rope<Int>.of([1, 2, 3])
        var plain = [1, 2, 3]
        for step in 0..60:
            let n = plain.len
            let kind = rand(0, 4)
            if kind == 0 or n == 0:
                let i = rand(0, n)
                r.insert(i, step)
                plain = [if j < i: plain[j] elif j == i: step else: plain[j - 1] for j in 0..=n]
            elif kind == 1:
                let i = rand(0, n - 1)
                assert r.remove(i) == plain[i]
                plain = [plain[j] for j in 0..n if j != i]
            elif kind == 2:
                let (a, b) = (rand(0, n), rand(0, n))
                let (lo, hi) = (min(a, b), max(a, b))
                r.reverse(lo..hi)
                plain = [if lo <= j < hi: plain[lo + hi - 1 - j] else: plain[j] for j in 0..n]
            elif kind == 3:
                let (a, b) = (rand(0, n), rand(0, n))
                let (lo, hi) = (min(a, b), max(a, b))
                let to = rand(0, n - (hi - lo))
                r.move_to(lo..hi, to)
                let cut = [plain[j] for j in lo..hi]
                let rest = [plain[j] for j in 0..n if not (lo <= j < hi)]
                plain = [if j < to: rest[j] elif j < to + cut.len: cut[j - to] else: rest[j - cut.len] for j in 0..n]
            else:
                let i = rand(0, n - 1)
                assert r.get(i) == plain[i]
            assert r.len == plain.len
        assert r.to_list() == plain

@test
fn a_text_editor_in_four_lines():
    var r = Rope<Char>.of([c for c in "hello world"])
    r.reverse(0..5)
    r.move_to(0..6, 5)
    r.insert(5, ' ')
    assert Str(r.to_list()) == "world olleh "
