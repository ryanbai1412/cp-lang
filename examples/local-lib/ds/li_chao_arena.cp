//! Many Li Chao trees in one pool of nodes: a hull per vertex of a tree, a hull per
//! layer of a DP, or every past version of one hull. Lines may be limited to a segment
//! of the coordinate range, and the range itself can be as wide as an `Int` allows,
//! because nodes are only created where a line actually lands.
use lib.ds.li_chao.{Line}

/// One set of lines inside a `LiChaoArena`. It is a plain handle: copying it is free,
/// and an old copy keeps answering for the lines it had when `added` made a new one.
pub struct Hull(node: ?Int)

/// A pool of hulls over the integer points `lo..=hi`. With `w = hi - lo`: `add` and
/// `min_at` are O(log w) and `add` makes at most one node; a line on a segment costs
/// O(log^2 w) time and O(log w) nodes; the persistent `added` copies the path it
/// touches, O(log w) nodes. For maxima, add `Line(-k, -m)` and negate the answer.
pub struct LiChaoArena(lo: Int, hi: Int):
    line: List<?Line>
    left: List<?Int>
    right: List<?Int>

    pub fn new(lo: Int, hi: Int) -> LiChaoArena:
        assert lo <= hi
        LiChaoArena(lo, hi, [], [], [])

    /// A hull with no lines. It takes no nodes.
    pub @get fn empty(self) -> Hull = Hull(none)

    /// How many nodes all the hulls use together.
    pub @get fn nodes(self) -> Int = line.len

    /// Forget every hull at once; handles made before this are no longer valid.
    pub fn clear(mut self):
        line.clear()
        left.clear()
        right.clear()

    /// Put `f` into `h` in place and return the handle to keep: `hull = pool.add(hull, f)`.
    /// Older copies of the handle see the new line too.
    pub fn add(mut self, h: Hull, f: Line) -> Hull = Hull(
        self.cover(h.node, lo, hi, lo, hi, f, false)
    )

    /// `add`, with `f` only counting for `x` in `from..=to`.
    pub fn add_on(mut self, h: Hull, from: Int, to: Int, f: Line) -> Hull:
        Hull(self.cover(h.node, lo, hi, max(from, lo), min(to, hi), f, false))

    /// A new hull with the lines of `h` and `f`; `h` itself is left as it was.
    pub fn added(mut self, h: Hull, f: Line) -> Hull = Hull(
        self.cover(h.node, lo, hi, lo, hi, f, true)
    )

    /// `added`, with `f` only counting for `x` in `from..=to`.
    pub fn added_on(mut self, h: Hull, from: Int, to: Int, f: Line) -> Hull:
        Hull(self.cover(h.node, lo, hi, max(from, lo), min(to, hi), f, true))

    /// The smallest `y` at `x` over the lines of `h` that count at `x`, or `none`.
    pub fn min_at(self, h: Hull, x: Int) -> ?Int:
        assert lo <= x <= hi
        var (at, l, r) = (h.node, lo, hi)
        var best: ?Int = none
        while at != none:
            let f = line[at!]
            if f != none and (best == none or f!.at(x) < best!): best = f!.at(x)
            let mid = (l + r) / 2
            if x <= mid: (at, r) = (left[at!], mid) else: (at, l) = (right[at!], mid + 1)
        best

    /// Every line of `from` put into `into`, in place: O(size of `from` * log w) on top
    /// of the lines' own segments. Merge the smaller hull into the larger one.
    pub fn merge(mut self, into: Hull, from: Hull) -> Hull:
        var out = into.node
        var todo = [(from.node, lo, hi)]
        while not todo.is_empty:
            let (at, l, r) = todo.pop()!
            if at == none: continue
            // a line stored at a node counts on that node's whole range
            if line[at!] != none: out = self.cover(out, lo, hi, l, r, line[at!]!, false)
            let mid = (l + r) / 2
            todo.push((left[at!], l, mid))
            todo.push((right[at!], mid + 1, r))
        Hull(out)

    /// Add `f` on `from..=to` below the node `at`, which spans `l..=r`. With `copy`,
    /// every node that would change is duplicated instead.
    fn cover(mut self, at: ?Int, l: Int, r: Int, from: Int, to: Int, f: Line, copy: Bool) -> ?Int:
        if to < l or r < from: return at
        if from <= l and r <= to: return self.settle(at, l, r, f, copy)
        let mid = (l + r) / 2
        let a = self.cover(if at == none: none else: left[at!], l, mid, from, to, f, copy)
        let b = self.cover(if at == none: none else: right[at!], mid + 1, r, from, to, f, copy)
        self.put(at, if at == none: none else: line[at!], a, b, copy)

    /// Add `f` on the whole of `l..=r`: keep the lower line at the midpoint here and
    /// send the other one to the half where it can still win.
    fn settle(mut self, at: ?Int, l: Int, r: Int, f: Line, copy: Bool) -> ?Int:
        if at == none or line[at!] == none:
            return self.put(
                at,
                f,
                if at == none: none else: left[at!],
                if at == none: none else: right[at!],
                copy,
            )
        let mid = (l + r) / 2
        var (keep, cur) = (line[at!]!, f)
        if cur.at(mid) < keep.at(mid): (keep, cur) = (cur, keep)
        var (a, b) = (left[at!], right[at!])
        if l < r:
            if cur.at(l) < keep.at(l): a = self.settle(a, l, mid, cur, copy)
            elif cur.at(r) < keep.at(r): b = self.settle(b, mid + 1, r, cur, copy)
        self.put(at, keep, a, b, copy)

    /// The node `at` with this content: written in place, or as a new node when `at`
    /// is missing or must stay untouched.
    fn put(mut self, at: ?Int, f: ?Line, a: ?Int, b: ?Int, copy: Bool) -> ?Int:
        if at != none and line[at!] == f and left[at!] == a and right[at!] == b: return at
        if at == none or copy:
            line.push(f)
            left.push(a)
            right.push(b)
            return line.len - 1
        (line[at!], left[at!], right[at!]) = (f, a, b)
        at

/// The same answer from the plain list of `(line, from, to)`.
fn slow_min(lines: List<(Line, Int, Int)>, x: Int) -> ?Int:
    var best: ?Int = none
    for (f, from, to) in lines:
        if from <= x <= to and (best == none or f.at(x) < best!): best = f.at(x)
    best

fn rand_line(span: Int) -> Line = Line(rand(-span, span), rand(-span * span, span * span))

@test
fn every_version_keeps_its_own_lines():
    for _ in 0..25:
        let (lo, hi) = (rand(-30, 0), rand(0, 30))
        var pool = LiChaoArena(lo, hi)
        var versions = [pool.empty]
        var lines: List<(Line, Int, Int)> = []
        for _ in 0..20:
            // every step makes a new version; all the old ones are checked below
            let f = rand_line(20)
            let (a, b) = (rand(lo - 3, hi + 3), rand(lo - 3, hi + 3))
            let (from, to) = if rand(0, 2) == 0: (lo, hi) else: (min(a, b), max(a, b))
            versions.push(pool.added_on(versions.last!, from, to, f))
            lines.push((f, from, to))
        for v in 0..versions.len:
            let have = [lines[i] for i in 0..v]
            for x in lo..=hi: assert pool.min_at(versions[v], x) == slow_min(have, x)

@test
fn a_forest_of_hulls_shares_one_pool():
    for _ in 0..25:
        let (lo, hi) = (rand(-25, 0), rand(0, 25))
        domain T = 0..5
        var pool = LiChaoArena(lo, hi)
        var hull = [T] _ => pool.empty
        var lines: [T]List<(Line, Int, Int)> = []
        var whole = 0
        for _ in 0..40:
            let t = T(rand(0, 4))
            let f = rand_line(15)
            if rand(0, 2) == 0:
                let (a, b) = (rand(lo, hi), rand(lo, hi))
                hull[t] = pool.add_on(hull[t], min(a, b), max(a, b), f)
                lines[t].push((f, min(a, b), max(a, b)))
            else:
                let before = pool.nodes
                hull[t] = pool.add(hull[t], f)
                lines[t].push((f, lo, hi))
                assert pool.nodes <= before + 1
        for t in T:
            for x in lo..=hi: assert pool.min_at(hull[t], x) == slow_min(lines[t], x)
        // fold every hull into the first one
        var all = hull[T(0)]
        var every = lines[T(0)]
        for t in T:
            if t == T(0): continue
            all = pool.merge(all, hull[t])
            every += lines[t]
        for x in lo..=hi: assert pool.min_at(all, x) == slow_min(every, x)
        pool.clear()
        assert pool.nodes == 0 and pool.min_at(pool.empty, lo) == none

@test
fn a_range_as_wide_as_the_coordinates():
    let big = 10 ** 9
    var pool = LiChaoArena(-big, big)
    var h = pool.empty
    var lines: List<(Line, Int, Int)> = []
    for _ in 0..60:
        let f = Line(rand(-big, big), rand(-big, big))
        let (a, b) = (rand(-big, big), rand(-big, big))
        if rand(0, 1) == 0:
            h = pool.add(h, f)
            lines.push((f, -big, big))
        else:
            h = pool.add_on(h, min(a, b), max(a, b), f)
            lines.push((f, min(a, b), max(a, b)))
    for x in [-big, big, 0, -1, 1] + [rand(-big, big) for _ in 0..200]:
        assert pool.min_at(h, x) == slow_min(lines, x)

@test
fn cheapest_jump_from_an_ancestor():
    // a rooted tree; reaching v from an ancestor u costs dp[u] + (h[v] - h[u]) ** 2.
    // Each vertex extends its parent's hull without disturbing it for its siblings.
    for _ in 0..25:
        let n = rand(1, 12)
        domain V = 0..n
        let parent = [V] v => if v.to_idx == 0: none else: V(rand(0, v.to_idx - 1))
        let h = [V] _ => rand(0, 30)
        var pool = LiChaoArena(0, 30)
        var hull = [V] _ => pool.empty
        var dp: [V]Int = 0
        for v in V:
            if parent[v] != none:
                dp[v] = pool.min_at(hull[parent[v]!], h[v])! + h[v] * h[v]
                hull[v] = hull[parent[v]!]
                var (want, u) = (Int.MAX, parent[v])
                while u != none:
                    want min= dp[u!] + (h[v] - h[u!]) ** 2
                    u = parent[u!]
                assert dp[v] == want
            hull[v] = pool.added(hull[v], Line(-2 * h[v], dp[v] + h[v] * h[v]))
