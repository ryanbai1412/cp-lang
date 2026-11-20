// APIO '23 P2 (Sequence): maximum frequency of a median over all subarrays.
// Sweep candidate values v = 1..=n. Each element is +1 (>v), 0 (==v), or -1 (<v);
// a SegTree over NP tracks prefix-balance bounds [yl, yr], net balance y, and count x
// with Tropical Optional identity NP(none, none, 0, 0). Between consecutive occurrences
// of v we extract block intervals (yl, yr, x), then maximize x_j - x_i over overlapping
// intervals (ChmaxMax LazySegTree) and two 2D dominance sweeps (max SegTree).
// original: corpus/submissions/5593302.cpp

struct NP:
    yl: ?Int = none
    yr: ?Int = none
    y: Int = 0
    x: Int = 0
    fn merge(self, o: NP) -> NP:
        NP(min(self.yl, self.y + o.yl), max(self.yr, self.y + o.yr), self.y + o.y, self.x + o.x)

type Block = (yl: Int, yr: Int, x: Int)

fn solve_overlap(blocks: List<Block>) -> Int:
    let ys = compress([b.yl for b in blocks] + [b.yr for b in blocks])
    let chmax = Action<?Int, ?Int>(
        id: none,
        compose: (old, nw) => max(old, nw),
        apply: (v, t) => max(v, t),
        over: max)
    var st = LazySegTree<ys.D, ?Int>(chmax, [ys.D] _ => none)
    var best = 0
    for b in blocks.rev:
        let r = ys.at(b.yl)..=ys.at(b.yr)
        best max= st.fold(r) - b.x
        st.apply(r, b.x)
    best

fn solve_dom(blocks: List<Block>) -> Int:
    let ys = compress([b.x - b.yl for b in blocks] + [b.x - b.yr for b in blocks])
    // Update at (yl, x - yl) with value x; query (yr, x - yr).
    // Equal x puts queries (is_q = true) before updates so yl == yr defers to solve_overlap.
    var evts: List<(key: Int, is_q: Bool, idx: Int)> = []
    for (i, b) in blocks.items:
        evts.push((b.yl, false, i))
        evts.push((b.yr, true, i))
    sort(mut evts, by: e => (desc(e.key), desc(e.is_q)))
    var st = SegTree<ys.D, ?Int>(max, [ys.D] _ => none)
    var cur: [ys.D]?Int = none
    var best = 0
    for e in evts:
        let b = blocks[e.idx]
        if not e.is_q:
            let y = ys.at(b.x - b.yl)
            cur[y] max= b.x
            st.set(y, cur[y])
        else:
            best max= st.fold(ys.at(b.x - b.yr)..) - b.x
    best

fn best_for_value(blocks: List<Block>) -> Int:
    let flipped = [(yl: -b.yr, yr: -b.yl, x: b.x) for b in blocks]
    max(solve_overlap(blocks), max(solve_dom(blocks), solve_dom(flipped)))

@grader
fn sequence(n: Int, a: [0..n]1..=n) -> Int:
    domain Val = 1..=n
    domain Pre = 0..=n
    var pos: [Val]List<Int> = []
    for i in 0..n: pos[Val.from_idx(a[i] - 1)!].push(i)

    var st = SegTree<Pre, NP>(
        Monoid(NP(), (l, r) => l.merge(r)),
        [Pre] p => if p == Pre.first: NP(0, 0, 0, 0) else: NP(1, 1, 1, 0))
    var ans = 0
    for v in Val:
        if pos[v].is_empty: continue
        for i in pos[v]: st.set(Pre.from_idx(i + 1)!, NP(0, 0, 0, 1))
        if pos[v].len > ans:
            var blocks: List<Block> = []
            var pref = NP(0, 0, 0, 0)
            fn add_gap(l: Int, r: Int):
                let seg = st.fold(Pre.from_idx(l)!..Pre.from_idx(r)!)
                blocks.push((seg.yl! + pref.y, seg.yr! + pref.y, pref.x + seg.x))
                pref = pref.merge(seg)
            add_gap(0, pos[v][0] + 1)
            for (p, q) in pos[v].adjacent: add_gap(p + 1, q + 1)
            add_gap(pos[v].last! + 1, n + 1)
            ans max= best_for_value(blocks)
        for i in pos[v]: st.set(Pre.from_idx(i + 1)!, NP(-1, -1, -1, 0))
    ans
