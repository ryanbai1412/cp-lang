//! Li Chao tree: a set of lines, queried for the lowest one at a point.
//! The convex hull trick without the requirement that slopes or queries
//! arrive sorted.

/// A line `y = k * x + m`.
pub struct Line(k: Int, m: Int):
    pub fn at(self, x: Int) -> Int = k * x + m

/// Lines over the integer points `lo..=hi`. `add` and `min_at` are
/// O(log(hi - lo)); memory is one node per `add` at most.
/// For maxima, add `Line(-k, -m)` and negate the answer.
pub struct LiChao(lo: Int, hi: Int):
    line: List<Line>
    left: List<?Int>
    right: List<?Int>

    pub fn new(lo: Int, hi: Int) -> LiChao:
        LiChao(lo, hi, [], [], [])

    pub fn add(mut self, fresh: Line):
        if line.is_empty:
            self.node(fresh)
            return
        var (at, l, h, cur) = (0, lo, hi, fresh)
        while true:
            let mid = (l + h) / 2
            // keep in this node whichever line is lower at mid; push the other one down
            if cur.at(mid) < line[at].at(mid):
                let old = line[at]
                line[at] = cur
                cur = old
            if l == h: return
            if cur.at(l) < line[at].at(l):
                if left[at] == none:
                    left[at] = self.node(cur)
                    return
                (at, h) = (left[at]!, mid)
            elif cur.at(h) < line[at].at(h):
                if right[at] == none:
                    right[at] = self.node(cur)
                    return
                (at, l) = (right[at]!, mid + 1)
            else:
                return

    /// The smallest `y` at `x` over all lines added, or `none` if there are none.
    pub fn min_at(self, x: Int) -> ?Int:
        assert lo <= x <= hi
        if line.is_empty: return none
        var (at, l, h) = (0, lo, hi)
        var best = line[0].at(x)
        while true:
            best min= line[at].at(x)
            let mid = (l + h) / 2
            let next = if x <= mid: left[at] else: right[at]
            if next == none: return best
            if x <= mid: h = mid else: l = mid + 1
            at = next!

    fn node(mut self, l: Line) -> Int:
        line.push(l)
        left.push(none)
        right.push(none)
        line.len - 1

@test
fn minimum_matches_scanning_every_line():
    for _ in 0..20:
        var tree = LiChao(-40, 40)
        var lines = List<Line>()
        assert tree.min_at(0) == none
        for _ in 0..25:
            let l = Line(rand(-9, 9), rand(-50, 50))
            lines.push(l)
            tree.add(l)
            for x in [-40, -17, -1, 0, 3, 22, 40]:
                assert tree.min_at(x) == min(m.at(x) for m in lines)

@test
fn a_dp_with_the_convex_hull_trick():
    // dp[i] = min over j < i of dp[j] + (h[i] - h[j])^2, the classic frog jump with squared cost
    let h = [rand(0, 30) for _ in 0..40]
    var slow = [0]
    for i in 1..h.len: slow.push(min(slow[j] + (h[i] - h[j]) ** 2 for j in 0..i))
    var tree = LiChao(0, 30)
    var fast = [0]
    for i in 0..h.len:
        if i > 0: fast.push(tree.min_at(h[i])! + h[i] * h[i])
        tree.add(Line(-2 * h[i], fast[i] + h[i] * h[i]))
    assert fast == slow
