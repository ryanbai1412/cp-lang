//! How much of a line is covered by at least one interval, under adding and removing
//! intervals: the structure behind "area of a union of rectangles" sweeps.

/// Cells `Y`, each with a width, and a multiset of ranges of cells. `add` is O(log n);
/// `covered` is O(1). A range may only be removed after it was added (counts stay
/// non-negative), which is what a sweep line does.
pub struct Cover<Y>:
    size: Int
    /// How many added ranges cover this whole node.
    cnt: List<Int>
    /// The covered width inside this node.
    len: List<Int>
    /// The total width of this node.
    width: List<Int>

    /// `width[y]` is the length of cell `y`: the gap between two neighbouring coordinates
    /// after compression, or 1 for unit cells.
    pub fn new(width: [Y]Int) -> Cover<Y>:
        var size = 1
        while size < Y.count: size *= 2
        var w = List.filled(2 * size, 0)
        for y in Y: w[size + y.to_idx] = width[y]
        for i in (1..size).rev: w[i] = w[2 * i] + w[2 * i + 1]
        Cover(size, List.filled(2 * size, 0), List.filled(2 * size, 0), w)

    /// Add `k` copies of the range `r` (`k = -1` removes one).
    pub fn add(mut self, r: Range<Y>, k: Int):
        self.walk(1, 0, size, r.l.ord, r.r.ord, k)

    /// The total width of the cells covered by at least one range.
    pub @get fn covered(self) -> Int = len[1]

    fn walk(mut self, node: Int, lo: Int, hi: Int, from: Int, to: Int, k: Int):
        if to <= lo or hi <= from: return
        if from <= lo and hi <= to:
            cnt[node] += k
        else:
            let mid = (lo + hi) / 2
            self.walk(2 * node, lo, mid, from, to, k)
            self.walk(2 * node + 1, mid, hi, from, to, k)
        len[node] = if cnt[node] > 0: width[node] elif node >= size: 0 else: len[2 * node] + len[2 * node + 1]

@test
fn covered_width_matches_counting_cells():
    domain Y = 0..13
    for _ in 0..25:
        let width = [Y] y => rand(1, 5)
        var c = Cover<Y>(width)
        var depth: [Y]Int = 0
        var live: List<Range<Y>> = []
        for _ in 0..30:
            if live.is_empty or rand(0, 2) > 0:
                let (a, b) = (Y.nth(rand(0, 12)), Y.nth(rand(0, 12)))
                let r: Range<Y> = if rand(0, 5) == 0: a.l..a.l else: min(a, b)..=max(a, b)
                live.push(r)
                c.add(r, 1)
                for y in r: depth[y] += 1
            else:
                let r = live.pop()!
                c.add(r, -1)
                for y in r: depth[y] -= 1
            assert c.covered == sum(width[y] for y in Y if depth[y] > 0)

@test
fn area_of_a_union_of_rectangles():
    // rectangles as (x1, x2, y1, y2) on unit cells 0..6
    domain Y = 0..6
    let rects = [(0, 3, 0, 2), (1, 5, 1, 4), (4, 6, 3, 6)]
    var c = Cover<Y>([Y] y => 1)
    var area = 0
    for x in 0..6:
        for (x1, x2, y1, y2) in rects:
            if x == x1: c.add(Y(y1).l..Y(y2 - 1).r, 1)
            if x == x2: c.add(Y(y1).l..Y(y2 - 1).r, -1)
        area += c.covered
    var want = 0
    for x in 0..6:
        for y in 0..6:
            if rects.any(((x1, x2, y1, y2)) => x1 <= x < x2 and y1 <= y < y2): want += 1
    assert area == want
