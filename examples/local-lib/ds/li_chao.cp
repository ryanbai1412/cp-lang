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

/// A line `y = k * x + m` paired with an arbitrary user payload `tag: V` (e.g. DP index/record).
pub struct TaggedLine<V>(k: Int, m: Int, tag: V):
    pub fn at(self, x: Int) -> Int = k * x + m

/// Li Chao tree returning both the minimum `y` and the winning `tag: V` at `x`.
pub struct LiChaoTag<V>(lo: Int, hi: Int):
    line: List<TaggedLine<V>>
    left: List<?Int>
    right: List<?Int>

    pub fn new(lo: Int, hi: Int) -> LiChaoTag<V>:
        LiChaoTag(lo, hi, [], [], [])

    pub fn add(mut self, fresh: TaggedLine<V>):
        if line.is_empty:
            self.node(fresh)
            return
        var (at, l, h, cur) = (0, lo, hi, fresh)
        while true:
            let mid = (l + h) / 2
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

    /// `(min_y, tag)` at `x`, or `none` if empty.
    pub fn min_at(self, x: Int) -> ?(Int, V):
        assert lo <= x <= hi
        if line.is_empty: return none
        var (at, l, h) = (0, lo, hi)
        var best = (line[0].at(x), line[0].tag)
        while true:
            let y = line[at].at(x)
            if y < best.0: best = (y, line[at].tag)
            let mid = (l + h) / 2
            let next = if x <= mid: left[at] else: right[at]
            if next == none: return best
            if x <= mid: h = mid else: l = mid + 1
            at = next!

    fn node(mut self, l: TaggedLine<V>) -> Int:
        line.push(l)
        left.push(none)
        right.push(none)
        line.len - 1

/// A floating-point line `y = k * x + m`.
pub struct LineF(k: F64, m: F64):
    pub fn at(self, x: F64) -> F64 = k * x + m

/// Continuous `F64` (`F64`) Li Chao tree over `[lo, hi]` with `iters` halving levels.
pub struct LiChaoF(lo: F64, hi: F64, iters: Int):
    line: List<LineF>
    left: List<?Int>
    right: List<?Int>

    pub fn new(lo: F64, hi: F64, iters: Int = 48) -> LiChaoF:
        LiChaoF(lo, hi, iters, [], [], [])

    pub fn add(mut self, fresh: LineF):
        if line.is_empty:
            self.node(fresh)
            return
        var (at, l, h, cur) = (0, lo, hi, fresh)
        for _ in 0..iters:
            let mid = (l + h) / 2.0
            if cur.at(mid) < line[at].at(mid):
                let old = line[at]
                line[at] = cur
                cur = old
            if cur.at(l) < line[at].at(l):
                if left[at] == none:
                    left[at] = self.node(cur)
                    return
                (at, h) = (left[at]!, mid)
            elif cur.at(h) < line[at].at(h):
                if right[at] == none:
                    right[at] = self.node(cur)
                    return
                (at, l) = (right[at]!, mid)
            else:
                return

    pub fn min_at(self, x: F64) -> ?F64:
        assert lo <= x and x <= hi
        if line.is_empty: return none
        var (at, l, h) = (0, lo, hi)
        var best = line[0].at(x)
        for _ in 0..iters:
            best min= line[at].at(x)
            let mid = (l + h) / 2.0
            let next = if x <= mid: left[at] else: right[at]
            if next == none: return best
            if x <= mid: h = mid else: l = mid
            at = next!
        best

    fn node(mut self, l: LineF) -> Int:
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

struct TagRec(id: Int, cost: Int)

@test
fn tagged_and_float_li_chao_match_brute_force():
    // Tagged LiChao returns the exact value and winning record tag
    var tagged = LiChaoTag<TagRec>(-20, 20)
    var lines: List<TaggedLine<TagRec>> = []
    for i in 0..15:
        let l = TaggedLine(rand(-5, 5), rand(-30, 30), TagRec(i, i * 10))
        lines.push(l)
        tagged.add(l)
    for x in -20..=20:
        let (y, rec) = tagged.min_at(x)!
        assert y == min(l.at(x) for l in lines)
        assert lines[rec.id].at(x) == y and rec.cost == rec.id * 10
    // Continuous F64 LiChao matches scanning over real points
    var flt = LiChaoF(-10.0, 10.0)
    var flines: List<LineF> = []
    for _ in 0..15:
        let l = LineF(F64(rand(-20, 20)) / 4.0, F64(rand(-40, 40)) / 4.0)
        flines.push(l)
        flt.add(l)
    for k in -20..=20:
        let x = F64(k) / 2.0
        let got = flt.min_at(x)!
        let want = min(l.at(x) for l in flines)
        assert abs(got - want) <= 1e-6
