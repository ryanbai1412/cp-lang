// Runnable today/ port of APIO '23 P2 (Sequence) for proto/cpc.
// Stdin protocol: n, then n integers a[0..n] in 1..=n.
// original: corpus/submissions/5593302.cpp

struct NP:
    has: Bool
    yl: Int
    yr: Int
    y: Int
    x: Int

fn np_merge(a: NP, b: NP) -> NP:
    if not a.has: return b
    if not b.has: return a
    return NP(true, min(a.yl, a.y + b.yl), max(a.yr, a.y + b.yr), a.y + b.y, a.x + b.x)

struct Block:
    yl: Int
    yr: Int
    x: Int

fn bsearch_idx(vals: List<Int>, x: Int) -> Int:
    var lo = 0
    var hi = vals.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if vals[mid] < x: lo = mid + 1
        else: hi = mid
    return lo

fn compress_list(raw: List<Int>) -> List<Int>:
    var s = raw
    s.sort()
    var out: List<Int> = []
    for x in s:
        if out.len == 0 or out[out.len - 1] != x:
            out.push(x)
    return out

// Range chmax / range max segment tree (no pushdown needed for associative chmax)
fn solve_overlap(blocks: List<Block>) -> Int:
    var raw: List<Int> = []
    for b in blocks:
        raw.push(b.yl)
        raw.push(b.yr)
    let ys = compress_list(raw)
    let m = ys.len
    var tree = [0..4 * m + 5] _ => -1
    var lazy = [0..4 * m + 5] _ => -1

    fn upd(o: Int, l: Int, r: Int, ql: Int, qr: Int, v: Int):
        tree[o] max= v
        if ql <= l and r <= qr:
            lazy[o] max= v
            return
        let mid = (l + r) / 2
        if ql <= mid: upd(o * 2, l, mid, ql, qr, v)
        if mid < qr: upd(o * 2 + 1, mid + 1, r, ql, qr, v)

    fn qry(o: Int, l: Int, r: Int, ql: Int, qr: Int) -> Int:
        if ql <= l and r <= qr: return tree[o]
        var res = lazy[o]
        let mid = (l + r) / 2
        if ql <= mid: res max= qry(o * 2, l, mid, ql, qr)
        if mid < qr: res max= qry(o * 2 + 1, mid + 1, r, ql, qr)
        return res

    var ans = 0
    var i = blocks.len - 1
    while i >= 0:
        let b = blocks[i]
        let yl = bsearch_idx(ys, b.yl)
        let yr = bsearch_idx(ys, b.yr)
        let got = qry(1, 0, m - 1, yl, yr)
        if got != -1: ans max= got - b.x
        upd(1, 0, m - 1, yl, yr, b.x)
        i -= 1
    return ans

fn solve_dom(blocks: List<Block>) -> Int:
    var raw: List<Int> = []
    for b in blocks:
        raw.push(b.x - b.yl)
        raw.push(b.x - b.yr)
    let ys = compress_list(raw)
    let m = ys.len
    domain Y = 0..m
    // Event: (-key, -is_q, idx) so larger key first; on tie queries (is_q=1) before updates (is_q=0)
    var evts: List<(Int, Int, Int)> = []
    for i in 0..blocks.len:
        let b = blocks[i]
        evts.push((-b.yl, 0, i))
        evts.push((-b.yr, -1, i))
    evts.sort()
    var st = SegTree<Y, Int>(-1, (a, b) => max(a, b))
    var cur: [Y]Int = -1
    var ans = 0
    for e in evts:
        let idx = e.2
        let b = blocks[idx]
        if e.1 == 0:
            let y = Y(bsearch_idx(ys, b.x - b.yl))
            cur[y] max= b.x
            st.set(y, cur[y])
        else:
            let y = Y(bsearch_idx(ys, b.x - b.yr))
            let got = st.query(y, Y.last)
            if got != -1: ans max= got - b.x
    return ans

fn best_for_value(blocks: List<Block>) -> Int:
    var ans = max(solve_overlap(blocks), solve_dom(blocks))
    var flipped: List<Block> = []
    for b in blocks:
        flipped.push(Block(-b.yr, -b.yl, b.x))
    ans max= solve_dom(flipped)
    return ans

fn main_solve():
    let n: Int = read()
    domain Val = 0..n
    domain Pre = 0..=n
    var pos: [Val]List<Int> = []
    for i in 0..n:
        let x: Int = read()
        pos[Val(x - 1)].push(i)

    let id_np = NP(false, 0, 0, 0, 0)
    var st = SegTree<Pre, NP>(id_np, (a, b) => np_merge(a, b))
    st.set(Pre(0), NP(true, 0, 0, 0, 0))
    for i in 1..=n:
        st.set(Pre(i), NP(true, 1, 1, 1, 0))

    var ans = 0
    for v in Val:
        if pos[v].len == 0: continue
        for i in pos[v]:
            st.set(Pre(i + 1), NP(true, 0, 0, 0, 1))
        if pos[v].len > ans:
            var blocks: List<Block> = []
            var pref = NP(true, 0, 0, 0, 0)
            fn add_gap(ql: Int, qr: Int):
                let seg = st.query(Pre(ql), Pre(qr - 1))
                blocks.push(Block(seg.yl + pref.y, seg.yr + pref.y, pref.x + seg.x))
                pref = np_merge(pref, seg)
            add_gap(0, pos[v][0] + 1)
            for k in 0..pos[v].len - 1:
                add_gap(pos[v][k] + 1, pos[v][k + 1] + 1)
            add_gap(pos[v][pos[v].len - 1] + 1, n + 1)
            ans max= best_for_value(blocks)
        for i in pos[v]:
            st.set(Pre(i + 1), NP(true, -1, -1, -1, 0))
    print(ans)

main_solve()
