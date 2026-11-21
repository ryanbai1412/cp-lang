// Runnable CP Lang v2.2 port of noi22p5 (NOI '22 P5 - Bubble Sort)
// Original C++: corpus/submissions/4832632.cpp
// Runs directly under the Rust `cpl run` and `cpl build` (c521b2c).
// Workarounds vs blind first draft (noi22p5_draft.cp):
// - No `SortedBag`, `LazySegTree`, `AddMin`, `compress`, or `desc` in Rust prelude -> compact userland structs
// - No `type Val = xs.D` or `for i in Pos if cond:` in Rust syntax -> explicit `Int` indices and `if`
// - Uses real v2.2 `@multitest`, `struct` methods, `List.sort(by:)`, `?Int`, `min=`, `??`

struct Cond:
    l: Int
    r: Int
    x: Int

struct Evt:
    kind: Int
    val: Int

struct Seg:
    l: Int
    r: Int

struct Fenwick:
    n: Int
    bit: List<Int>

    fn make(n: Int) -> Fenwick:
        var bit: List<Int> = []
        for _ in 0..(n + 2):
            bit.push(0)
        Fenwick(n:, bit:)

    fn add(mut self, i_in: Int, delta: Int):
        var i = i_in + 1
        while i <= self.n:
            self.bit[i] += delta
            i += i & (-i)

    fn sum_prefix(self, i_in: Int) -> Int:
        var s = 0
        var i = i_in
        while i > 0:
            s += self.bit[i]
            i -= i & (-i)
        s

struct LazyMinTree:
    n: Int
    v: List<Int>
    lz: List<Int>

    fn make(n: Int) -> LazyMinTree:
        var v: List<Int> = []
        var lz: List<Int> = []
        let sz = 4 * n + 8
        for _ in 0..sz:
            v.push(0)
            lz.push(0)
        LazyMinTree(n:, v:, lz:)

    fn upd(mut self, o: Int, l: Int, r: Int, ql: Int, qr: Int, x: Int):
        if ql <= l and r <= qr:
            self.v[o] += x
            self.lz[o] += x
            return
        let mid = (l + r) / 2
        let lc = o * 2
        let rc = lc + 1
        if ql <= mid:
            self.upd(lc, l, mid, ql, qr, x)
        if mid < qr:
            self.upd(rc, mid + 1, r, ql, qr, x)
        self.v[o] = min(self.v[lc], self.v[rc]) + self.lz[o]

    fn apply_half_open(mut self, l: Int, r: Int, x: Int):
        if l >= r or l >= self.n:
            return
        let qr = min(r - 1, self.n - 1)
        if l <= qr:
            self.upd(1, 0, self.n - 1, l, qr, x)

    fn qry(self, o: Int, l: Int, r: Int, ql: Int, qr: Int) -> ?Int:
        if ql <= l and r <= qr:
            return self.v[o]
        let mid = (l + r) / 2
        let lc = o * 2
        let rc = lc + 1
        var ans: ?Int = none
        if ql <= mid:
            ans min= self.qry(lc, l, mid, ql, qr)
        if mid < qr:
            ans min= self.qry(rc, mid + 1, r, ql, qr)
        ans + self.lz[o]

    fn fold_half_open(self, l: Int, r: Int) -> Int:
        if l >= r or l >= self.n:
            return 0
        let qr = min(r - 1, self.n - 1)
        if l > qr:
            return 0
        self.qry(1, 0, self.n - 1, l, qr) ?? 0

fn unique_sorted(xs_in: List<Int>) -> List<Int>:
    let xs = xs_in.sorted()
    var out: List<Int> = []
    for x in xs:
        if out.len == 0 or out.last! != x:
            out.push(x)
    out

fn lower_bound(xs: List<Int>, target: Int) -> Int:
    let lo = xs.first_ge(target) ?? xs.len
    lo

fn find_next(mut par: List<Int>, x: Int) -> Int:
    var r = x
    while par[r] != r:
        r = par[r]
    var cur = x
    while par[cur] != r:
        let nxt = par[cur]
        par[cur] = r
        cur = nxt
    r

fn solve_case():
    let n: Int = read()
    let m_conds: Int = read()
    var conds: List<Cond> = []
    var raw_vals: List<Int> = []
    for _ in 0..m_conds:
        let l: Int = read()
        let r: Int = read()
        let x: Int = read()
        conds.push(Cond(l: l - 1, r: r - 1, x:))
        raw_vals.push(x)

    let uniq = unique_sorted(raw_vals)
    let m = uniq.len
    if m == 0:
        print(0)
        return

    var evts: List<List<Evt>> = []
    for _ in 0..(n + 1):
        var empty: List<Evt> = []
        evts.push(empty)
    var segs: List<List<Seg>> = []
    for _ in 0..m:
        var empty: List<Seg> = []
        segs.push(empty)

    for c in conds:
        let cx = lower_bound(uniq, c.x)
        evts[c.l].push(Evt(kind: 1, val: cx))
        evts[c.r + 1].push(Evt(kind: -1, val: cx))
        segs[cx].push(Seg(l: c.l, r: c.r))

    // Sweep with active multiset (modeled via Map<Int, Int> counts + max key)
    var counts: Map<Int, Int> = []
    counts[0] = 1
    var arr: List<Int> = []
    var eqs: List<Bool> = []
    for i in 0..n:
        for e in evts[i]:
            if e.kind == 1:
                let prev = counts.get(e.val) ?? 0
                counts[e.val] = prev + 1
            else:
                let prev = counts.get(e.val) ?? 0
                if prev <= 1:
                    let _ = counts.remove(e.val)
                else:
                    counts[e.val] = prev - 1
        var mx = 0
        for (k, _) in counts:
            mx = k
        arr.push(mx)
        eqs.push(false)

    var inds: List<List<Int>> = []
    for _ in 0..m:
        var empty: List<Int> = []
        inds.push(empty)
    for i in 0..n:
        inds[arr[i]].push(i)

    for v in 0..m:
        // Sort segments descending by (l, r) via negative composite key
        segs[v].sort(by: s => -(s.l * (n + 1) + s.r))
        let vs = inds[v]
        let sz = vs.len
        var nxt_free: List<Int> = []
        for idx in 0..(sz + 1):
            nxt_free.push(idx)
        var mni: ?Int = none
        for s in segs[v]:
            if mni != none and s.l <= (mni ?? -1) and (mni ?? -1) <= s.r:
                continue
            let start = lower_bound(vs, s.l)
            let slot = find_next(mut nxt_free, start)
            if slot >= sz or vs[slot] > s.r:
                print(-1)
                return
            let chosen = vs[slot]
            mni = chosen
            eqs[chosen] = true
            nxt_free[slot] = slot + 1

    var ans = 0
    var bit = Fenwick.make(m)
    for i in (0..n).rev:
        if eqs[i]:
            ans += bit.sum_prefix(arr[i])
            bit.add(arr[i], 1)

    var st = LazyMinTree.make(m)
    for i in 0..n:
        if eqs[i]:
            st.apply_half_open(arr[i] + 1, m, 1)
    for i in 0..n:
        st.apply_half_open(0, arr[i], 1)
        if eqs[i]:
            st.apply_half_open(arr[i] + 1, m, -1)
        else:
            ans += st.fold_half_open(arr[i], m)
    print(ans)

let tcs: Int = read()
for _ in 0..tcs:
    solve_case()
