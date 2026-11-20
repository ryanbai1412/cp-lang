//! A segment tree over a huge integer range that only creates the nodes it touches.

/// Point additions and range sums over the integers `lo..hi`, where every point starts at 0.
/// Use it when coordinates are too wide to compress offline. With `w = hi - lo`, `add` and
/// `sum` are O(log w), and each `add` creates at most O(log w) nodes.
pub struct SparseSegTree(lo: Int, hi: Int):
    total: List<Int>
    left: List<?Int>
    right: List<?Int>

    pub fn new(lo: Int, hi: Int) -> SparseSegTree:
        assert lo < hi
        SparseSegTree(lo, hi, [0], [none], [none])

    /// Add `d` to the value at `x`.
    pub fn add(mut self, x: Int, d: Int):
        assert lo <= x and x < hi
        self.put(0, lo, hi, x, d)

    /// The sum of the values at `from..to`; zero when the range is empty.
    pub fn sum(self, from: Int, to: Int) -> Int = self.fold(0, lo, hi, max(from, lo), min(to, hi))

    /// How many nodes exist.
    pub @get fn nodes(self) -> Int = total.len

    fn put(mut self, node: Int, l: Int, r: Int, x: Int, d: Int):
        total[node] += d
        if r - l == 1: return
        let mid = l + (r - l) / 2
        if x < mid:
            left[node] ??= self.make()
            self.put(left[node]!, l, mid, x, d)
        else:
            right[node] ??= self.make()
            self.put(right[node]!, mid, r, x, d)

    fn make(mut self) -> Int:
        total.push(0)
        left.push(none)
        right.push(none)
        total.len - 1

    fn fold(self, node: Int, l: Int, r: Int, a: Int, b: Int) -> Int:
        if a >= b or b <= l or r <= a: return 0
        if a <= l and r <= b: return total[node]
        let mid = l + (r - l) / 2
        var s = 0
        if let c = left[node]: s += self.fold(c, l, mid, a, b)
        if let c = right[node]: s += self.fold(c, mid, r, a, b)
        s

@test
fn matches_a_plain_array_on_a_small_range():
    for _ in 0..30:
        var st = SparseSegTree(-20, 20)
        var plain = List.filled(40, 0)
        for _ in 0..60:
            let x = rand(-20, 19)
            let d = rand(-5, 5)
            st.add(x, d)
            plain[x + 20] += d
            let a = rand(-25, 25)
            let b = rand(-25, 25)
            assert st.sum(a, b) == sum(plain[i + 20] for i in max(a, -20)..min(b, 20))

@test
fn stays_small_on_a_huge_range():
    let big = 1_000_000_000_000_000_000
    var st = SparseSegTree(-big, big)
    let points = [(-big, 3), (big - 1, 4), (0, -2), (12_345_678_901, 7)]
    for (x, d) in points: st.add(x, d)
    assert st.sum(-big, big) == 12
    assert st.sum(0, big) == 9 and st.sum(-big, 0) == 3 and st.sum(1, 1) == 0
    assert st.nodes <= 4 * 62 + 1
