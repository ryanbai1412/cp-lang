// Runnable CP Lang v2.2 port of bsspc21s4 (BSSPC '21 S4 - James's Rectangles)
// Original C++: corpus/submissions/4152932.cpp
// Runs directly under the Rust `cpl run` and `cpl build` (c521b2c).
// Workarounds vs blind first draft (bsspc21s4_draft.cp):
// - No `;` statement separator outside `{ ... }`
// - No path-dependent `type X = cx.D` or `compress(xs)` in Rust prelude -> inline dedup + binary search
// - Explicit `-> ()` on recursive void methods (workaround for cpl-ir E0204 when called before definition)
// - Uses real v2.2 `struct` methods, `ctz`, `?Int`, `min=`, `??`

fn ctz_safe(x: Int) -> Int:
    if x == 0:
        return 100
    ctz(x)

struct Node:
    ans: Int
    l: Int
    r: Int
    l_int: Int
    r_int: ?Int
    l_type: Bool
    r_type: Bool

    fn single(l: Int, r: Int, kind: Bool) -> Node:
        Node(ans: 100, l: l, r: r, l_int: r, r_int: none, l_type: kind, r_type: false)

    fn is_one(self) -> Bool:
        self.r_int == none

    fn merge(self, o: Node) -> Node:
        var ans = min(self.ans, o.ans)
        let l = self.l
        let r = o.r
        var l_int = self.l_int
        var r_int = self.r_int
        let l_type = self.l_type
        var r_type = self.r_type

        if self.is_one():
            if o.is_one():
                if l_type == o.l_type:
                    l_int = r
                else:
                    r_int = o.l
                    r_type = o.l_type
            else:
                if l_type == o.l_type:
                    l_int = o.l_int
                    r_int = o.r_int
                    r_type = o.r_type
                else:
                    ans min= ctz_safe(o.l)
                    ans min= ctz_safe(o.l_int)
                    r_int = o.r_int
                    r_type = o.r_type
        else:
            if o.is_one():
                if r_type != o.l_type:
                    r_int = o.l
                    r_type = o.l_type
            else:
                let ri = r_int ?? 0
                if r_type == o.l_type:
                    ans min= ctz_safe(ri)
                    ans min= ctz_safe(o.l_int)
                else:
                    ans min= ctz_safe(ri)
                    ans min= ctz_safe(o.l)
                    ans min= ctz_safe(o.l_int)
                r_int = o.r_int
                r_type = o.r_type
        Node(ans: ans, l: l, r: r, l_int: l_int, r_int: r_int, l_type: l_type, r_type: r_type)

    fn compute(self) -> Int:
        if self.is_one():
            return min(ctz_safe(self.l), ctz_safe(self.r))
        var best = self.ans
        best min= ctz_safe(self.l_int)
        best min= ctz_safe(self.r_int ?? 0)
        best min= ctz_safe(self.l)
        best min= ctz_safe(self.r)
        best

struct CoverTree:
    n: Int
    leaves: List<Node>
    tree: List<Node>
    cover: List<Int>

    fn maintain(mut self, o: Int, l: Int, r: Int):
        let base = if l == r: self.leaves[l] else: self.tree[o * 2].merge(self.tree[o * 2 + 1])
        if self.cover[o] > 0:
            self.tree[o] = Node.single(base.l, base.r, true)
        else:
            self.tree[o] = base

    fn build(mut self, o: Int, l: Int, r: Int) -> ():
        if l < r:
            let mid = (l + r) / 2
            self.build(o * 2, l, mid)
            self.build(o * 2 + 1, mid + 1, r)
        self.maintain(o, l, r)

    fn make(leaves: List<Node>) -> CoverTree:
        let n = leaves.len
        let sz = 4 * n + 8
        var tree: List<Node> = []
        var cover: List<Int> = []
        let dummy = Node.single(0, 0, false)
        for _ in 0..sz:
            tree.push(dummy)
            cover.push(0)
        var st = CoverTree(n: n, leaves: leaves, tree: tree, cover: cover)
        if n > 0:
            st.build(1, 0, n - 1)
        st

    fn update_rec(mut self, o: Int, l: Int, r: Int, ql: Int, qr: Int, delta: Int) -> ():
        if ql <= l and r <= qr:
            self.cover[o] += delta
            assert self.cover[o] >= 0
        else:
            let mid = (l + r) / 2
            if ql <= mid:
                self.update_rec(o * 2, l, mid, ql, qr, delta)
            if mid < qr:
                self.update_rec(o * 2 + 1, mid + 1, r, ql, qr, delta)
        self.maintain(o, l, r)

    fn apply(mut self, ql: Int, qr: Int, delta: Int):
        if ql <= qr and self.n > 0:
            self.update_rec(1, 0, self.n - 1, ql, qr, delta)

    fn all(self) -> Node:
        self.tree[1]

struct Rect:
    x1: Int
    y1: Int
    x2: Int
    y2: Int

struct Evt:
    l: Int
    r: Int
    delta: Int

fn unique_sorted(xs_in: List<Int>) -> List<Int>:
    let xs = xs_in.sorted()
    var out: List<Int> = []
    for x in xs:
        if out.len == 0 or out[out.len - 1] != x:
            out.push(x)
    out

fn lower_bound(xs: List<Int>, target: Int) -> Int:
    var lo = 0
    var hi = xs.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if xs[mid] < target:
            lo = mid + 1
        else:
            hi = mid
    lo

fn sweep(rects: List<Rect>) -> Int:
    if rects.len == 0:
        return 100
    var xs_raw: List<Int> = []
    var ys_raw: List<Int> = []
    for r in rects:
        xs_raw.push(r.x1)
        xs_raw.push(r.x2)
        ys_raw.push(r.y1)
        ys_raw.push(r.y2)
    let xs = unique_sorted(xs_raw)
    let ys = unique_sorted(ys_raw)
    let nx = xs.len
    let ny = ys.len
    if ny < 2:
        return 100

    var events: List<List<Evt>> = []
    for _ in 0..(nx + 1):
        var empty: List<Evt> = []
        events.push(empty)
    for r in rects:
        let x1 = lower_bound(xs, r.x1)
        let x2 = lower_bound(xs, r.x2)
        let y1 = lower_bound(ys, r.y1)
        let y2 = lower_bound(ys, r.y2)
        events[x1].push(Evt(l: y1, r: y2 - 1, delta: 1))
        events[x2].push(Evt(l: y1, r: y2 - 1, delta: -1))

    var leaves: List<Node> = []
    for i in 0..(ny - 1):
        leaves.push(Node.single(ys[i], ys[i + 1], false))
    var st = CoverTree.make(leaves)
    var ans = 100
    for i in 0..=nx:
        for e in events[i]:
            st.apply(e.l, e.r, e.delta)
        ans min= st.all().compute()
    ans

let n: Int = read()
let m: Int = read()
let k: Int = read()
var rects: List<Rect> = []
var swapped: List<Rect> = []
for _ in 0..k:
    let x1: Int = read()
    let y1: Int = read()
    let x2: Int = read()
    let y2: Int = read()
    rects.push(Rect(x1: x1 - 1, y1: y1 - 1, x2: x2, y2: y2))
    swapped.push(Rect(x1: y1 - 1, y1: x1 - 1, x2: y2, y2: x2))

var ans = min(ctz_safe(n), ctz_safe(m))
ans min= sweep(rects)
ans min= sweep(swapped)
print(ans)
