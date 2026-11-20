// ETSR '23 Round 1 P4 — Segment Tree Beats (Floor Division) (CP Lang v2 spec)
// original: corpus/submissions/5495163.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Range add, range Euclidean floor-division (`/`), range min, and range sum.
// CP Lang's built-in Euclidean `/` replaces C++'s custom `divf` helper.

struct Node:
    mn: Int
    mx: Int
    sum: Int

    fn merge(self, o: Node) -> Node:
        Node(min(self.mn, o.mn), max(self.mx, o.mx), self.sum + o.sum)

struct Tag:
    set: ?Int = none
    add: Int = 0

    fn compose(self, newer: Tag) -> Tag:
        if newer.set != none: newer
        else: Tag(self.set, self.add + newer.add)

    fn apply(self, node: Node, len: Int) -> Node:
        if let s = self.set:
            let v = s + self.add
            Node(v, v, len * v)
        else:
            Node(node.mn + self.add, node.mx + self.add, node.sum + len * self.add)

// FRICTION ET2: `LazySegTree` applies tags unconditionally and cannot inspect
// node `(mn, mx)` to decide whether to stop or recurse (Segment Tree Beats).
// We therefore write a compact recursive Beats tree directly.
struct BeatsTree:
    n: Int
    t: List<Node>
    lz: List<Tag>

    fn new(a: List<Int>) -> BeatsTree:
        let n = a.len
        var bt = BeatsTree(n, List.filled(4 * n, Node(0, 0, 0)), List.filled(4 * n, Tag()))
        bt.build(1, 0, n - 1, a)
        bt

    fn build(mut self, o: Int, l: Int, r: Int, a: List<Int>):
        if l == r:
            self.t[o] = Node(a[l], a[l], a[l])
            return
        let mid = (l + r) / 2
        self.build(2 * o, l, mid, a)
        self.build(2 * o + 1, mid + 1, r, a)
        self.t[o] = self.t[2 * o].merge(self.t[2 * o + 1])

    fn put(mut self, o: Int, l: Int, r: Int, tag: Tag):
        self.t[o] = tag.apply(self.t[o], r - l + 1)
        self.lz[o] = self.lz[o].compose(tag)

    fn push(mut self, o: Int, l: Int, r: Int):
        let mid = (l + r) / 2
        self.put(2 * o, l, mid, self.lz[o])
        self.put(2 * o + 1, mid + 1, r, self.lz[o])
        self.lz[o] = Tag()

    fn range_add(mut self, o: Int, l: Int, r: Int, ql: Int, qr: Int, x: Int):
        if ql <= l and r <= qr:
            self.put(o, l, r, Tag(none, x))
            return
        self.push(o, l, r)
        let mid = (l + r) / 2
        if ql <= mid: self.range_add(2 * o, l, mid, ql, qr, x)
        if mid < qr: self.range_add(2 * o + 1, mid + 1, r, ql, qr, x)
        self.t[o] = self.t[2 * o].merge(self.t[2 * o + 1])

    fn range_div(mut self, o: Int, l: Int, r: Int, ql: Int, qr: Int, x: Int):
        if ql <= l and r <= qr:
            let cmn = self.t[o].mn / x
            let cmx = self.t[o].mx / x
            if cmn == cmx:
                self.put(o, l, r, Tag(cmn, 0))
                return
            elif self.t[o].mx - self.t[o].mn == 1:
                self.put(o, l, r, Tag(none, cmx - self.t[o].mx))
                return
        self.push(o, l, r)
        let mid = (l + r) / 2
        if ql <= mid: self.range_div(2 * o, l, mid, ql, qr, x)
        if mid < qr: self.range_div(2 * o + 1, mid + 1, r, ql, qr, x)
        self.t[o] = self.t[2 * o].merge(self.t[2 * o + 1])

    fn query(mut self, o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Node:
        if ql <= l and r <= qr: return self.t[o]
        self.push(o, l, r)
        let mid = (l + r) / 2
        if qr <= mid: self.query(2 * o, l, mid, ql, qr)
        elif ql > mid: self.query(2 * o + 1, mid + 1, r, ql, qr)
        else: self.query(2 * o, l, mid, ql, qr).merge(self.query(2 * o + 1, mid + 1, r, ql, qr))

let (n, q): (1..=1e5, 1..=1e5) = read()
domain I = 1..=n
let init: [I]Int = read()
var st = BeatsTree.new([init[i] for i in I])

for _ in 0..q:
    let (ty, l, r): (1..=4, I, I) = read()
    match ty:
        1 =>
            let x: Int = read()
            st.range_add(1, 0, n - 1, l.to_idx, r.to_idx, x)
        2 =>
            let x: Int = read()
            st.range_div(1, 0, n - 1, l.to_idx, r.to_idx, x)
        3 => print(st.query(1, 0, n - 1, l.to_idx, r.to_idx).mn)
        4 => print(st.query(1, 0, n - 1, l.to_idx, r.to_idx).sum)
