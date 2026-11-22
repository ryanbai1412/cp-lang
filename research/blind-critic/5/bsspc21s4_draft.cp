// Problem: bsspc21s4 (BSSPC '21 S4 - 2D Rectangle Sweep / Cover-Count SegTree)
// Original: corpus/submissions/4152932.cpp
// Blind Critic #5 — First Draft (strictly from docs/one-page-cheat-sheet.md v2.2)

fn ctz_safe(x: Int) -> Int:
    if x == 0: 100 else: ctz(x)

struct Node:
    ans: Int = 100
    l: Int = 0
    r: Int = 0
    l_int: Int = 0
    r_int: ?Int = none
    l_type: Bool = false
    r_type: Bool = false

    fn is_one(self) -> Bool:
        return self.r_int == none

    fn single(l: Int, r: Int, kind: Bool) -> Node:
        return Node(ans: 100, l:, r:, l_int: r, r_int: none, l_type: kind, r_type: false)

    fn merge(self, o: Node) -> Node:
        var res = self
        res.r = o.r
        res.ans min= o.ans
        fn upd(cl: Int, cr: Int):
            res.ans min= ctz_safe(cl)
            res.ans min= ctz_safe(cr)
        if res.is_one():
            if o.is_one():
                if res.l_type == o.l_type:
                    res.l_int = res.r
                else:
                    res.r_int = o.l
                    res.r_type = o.l_type
            else:
                if res.l_type == o.l_type:
                    res.l_int = o.l_int
                    res.r_int = o.r_int
                    res.r_type = o.r_type
                else:
                    upd(o.l, o.l_int)
                    res.r_int = o.r_int
                    res.r_type = o.r_type
        else:
            if o.is_one():
                if res.r_type != o.l_type:
                    res.r_int = o.l
                    res.r_type = o.l_type
            else:
                if res.r_type == o.l_type:
                    upd(res.r_int!, o.l_int)
                else:
                    upd(res.r_int!, o.l)
                    upd(o.l, o.l_int)
                res.r_int = o.r_int
                res.r_type = o.r_type
        return res

    fn compute(self) -> Int:
        if self.is_one():
            return min(ctz_safe(self.l), ctz_safe(self.r))
        return min([self.ans, ctz_safe(self.l_int), ctz_safe(self.r_int!), ctz_safe(self.l), ctz_safe(self.r)])

// Note: LazySegTree<D, Node>(Action(0, (nf, of) => nf + of, (f, val, _) => if f > 0: Node.single(val.l, val.r, true) else: val, over: ...), leaves)
// CANNOT implement non-pushdown rectangle cover-count (+1/-1 on overlapping ranges),
// because push-down splits a parent's +1 across children and a later -1 cannot tell cover 1 -> 0 from 2 -> 1.
// Instead we write a tag-permanent (non-pushdown) recursive segment tree using Range<D>.mid and (r.l..m.r, m.r..r.r).
struct CoverTree<D>:
    leaves: [D]Node
    tree: List<Node>
    cover: List<Int>
    full: Range<D>

    fn new(leaves: [D]Node) -> CoverTree<D>:
        let sz = 4 * D.K + 5
        var st = CoverTree<D>(leaves:, tree: List.filled(sz, Node()), cover: List.filled(sz, 0), full: D.first..=D.last)
        st.build(1, st.full)
        return st

    fn maintain(mut self, o: Int, r: Range<D>):
        let base = if r.len == 1: self.leaves[r.first!] else: self.tree[o * 2].merge(self.tree[o * 2 + 1])
        self.tree[o] = if self.cover[o] > 0: Node.single(base.l, base.r, true) else: base

    fn build(mut self, o: Int, r: Range<D>):
        if r.len > 1:
            let m = r.mid!
            self.build(o * 2, r.l..m.r)
            self.build(o * 2 + 1, m.r..r.r)
        self.maintain(o, r)

    fn update_rec(mut self, o: Int, r: Range<D>, q: Range<D>, delta: Int):
        if q.l <= r.l and r.r <= q.r:
            self.cover[o] += delta
            assert self.cover[o] >= 0
        else:
            let m = r.mid!
            let (left, right) = (r.l..m.r, m.r..r.r)
            if q.l < left.r and left.l < q.r:
                self.update_rec(o * 2, left, q, delta)
            if q.l < right.r and right.l < q.r:
                self.update_rec(o * 2 + 1, right, q, delta)
        self.maintain(o, r)

    fn apply(mut self, q: Range<D>, delta: Int):
        if not q.is_empty:
            self.update_rec(1, self.full, q, delta)

    fn all(self) -> Node:
        return self.tree[1]

type Rect = (x1: Int, y1: Int, x2: Int, y2: Int)

fn sweep(rects: List<Rect>) -> Int:
    var xs_raw: List<Int> = []
    var ys_raw: List<Int> = []
    for r in rects:
        xs_raw.push(r.x1); xs_raw.push(r.x2)
        ys_raw.push(r.y1); ys_raw.push(r.y2)
    let cx = compress(xs_raw)
    let cy = compress(ys_raw)
    type X = cx.D
    domain YSeg = 0..(cy.D.K - 1)

    var events: [X]List<(YSeg, YSeg, Int)> = []
    for r in rects:
        let yl = YSeg.from_idx(cy.at(r.y1).to_idx)!
        let yr = YSeg.from_idx(cy.at(r.y2).to_idx - 1)!
        events[cx.at(r.x1)].push((yl, yr, 1))
        events[cx.at(r.x2)].push((yl, yr, -1))

    let leaves: [YSeg]Node = [YSeg] i => Node.single(cy[cy.D.from_idx(i.to_idx)!], cy[cy.D.from_idx(i.to_idx + 1)!], false)
    var st = CoverTree<YSeg>.new(leaves)
    var ans = 100
    for x in X:
        for (l, r, d) in events[x]:
            st.apply(l..=r, d)
        ans min= st.all().compute()
    return ans

let (n, m, k): (Int, Int, Int) = read()
var rects: List<Rect> = []
for _ in 0..k:
    let (x1, y1, x2, y2): (Int, Int, Int, Int) = read()
    rects.push((x1 - 1, y1 - 1, x2, y2))

var ans = min(ctz_safe(n), ctz_safe(m))
ans min= sweep(rects)
let swapped = [(r.y1, r.x1, r.y2, r.x2) for r in rects]
ans min= sweep(swapped)
print(ans)
