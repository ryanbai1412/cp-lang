// expect: ok, output "ok"
// input: "1 300\n"
// model: OnlineLis
// model: MonoStack
// model: cartesian_tree
// model: valleys
// model: spans
// model: prev_smaller
// model: next_smaller
// model: prev_same
// model: next_same
// model: outermost
// model: maximal
// model: minimal
// model: ring
// model: Ring
// model: RingSlice
use std.rand.{Rng}
use std.seq.{MonoStack, OnlineLis}

fn lis_end(v: List<Int>, i: Int, strict: Bool) -> Int:
    var best = 1
    for j in 0..i:
        if (v[j] < v[i] or (not strict and v[j] == v[i])) and lis_end(v, j, strict) + 1 > best: best = lis_end(v, j, strict) + 1
    best

fn cart(a: List<Int>, lo: Int, hi: Int, min_root: Bool, mut par: List<Int>, up: Int) -> Int:
    if lo >= hi: return -1
    var m = lo
    for j in lo..hi:
        if (min_root and a[j] < a[m]) or (not min_root and a[j] > a[m]): m = j
    par[m] = up
    let _ = cart(a, lo, m, min_root, mut par, m)
    let _ = cart(a, m + 1, hi, min_root, mut par, m)
    m

fn opt(x: ?Int) -> Int = x ?? -1

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var lis_strict = rng.int(0, 1) == 1
var lis = OnlineLis<Int>(strict: lis_strict)
var pushed: List<Int> = []
var marks: List<Int> = []
let keep_smaller = rng.int(0, 1) == 1
let ms_strict = rng.int(0, 1) == 1
var ms = MonoStack<Int, Int>(keep_smaller:, strict: ms_strict)
var st: List<(Int, Int)> = []
var st_log: List<List<(Int, Int)>> = []
var ms_marks: List<(Int, List<(Int, Int)>)> = []
var bad = ""
for op in 0..n:
    var why = ""
    let k = rng.int(0, 9)
    if k == 0:
        lis_strict = rng.int(0, 1) == 1
        lis = OnlineLis<Int>(strict: lis_strict)
        pushed = []
        marks = []
    elif k <= 3 and pushed.len < 9:
        let x = rng.int(0, 4)
        pushed.push(x)
        if lis.push(x) != lis_end(pushed, pushed.len - 1, lis_strict): why = "OnlineLis push {x} after {pushed}"
    elif k == 4:
        if rng.int(0, 1) == 0:
            marks.push(lis.mark())
        elif marks.len > 0:
            let m = marks.pop()!
            lis.undo_to(m)
            pushed = [pushed[j] for j in 0..m]
        else:
            lis.clear()
            pushed = []
    elif k <= 7:
        let i = op
        let x = rng.int(0, 4)
        var popped: List<(Int, Int)> = []
        while st.len > 0 and (if st.last!.1 == x: ms_strict elif keep_smaller: st.last!.1 < x else: st.last!.1 > x):
            popped.push(st.pop()!)
        let below = if st.len > 0: Some(st.last!.0) else: None
        st.push((i, x))
        if ms.push(i, x) != below: why = "MonoStack push {x}"
    elif k == 8:
        if rng.int(0, 1) == 0:
            ms_marks.push((ms.mark(), st.clone()))
        elif ms_marks.len > 0:
            let (m, saved) = ms_marks.pop()!
            ms.undo_to(m)
            st = saved
        else:
            ms.clear()
            st = []
            ms_marks = []
    else:
        let a = [rng.int(0, 4) for _ in 0..rng.int(0, 7)]
        let len = a.len
        let min_root = rng.int(0, 1) == 1
        var par = [-1 for _ in 0..len]
        let root = cart(a, 0, len, min_root, mut par, -1)
        let ct = a.cartesian_tree(min_root:)
        let ct_ok = opt(ct.root) == root and (0..len).all(i => opt(ct.parent[i]) == par[i]) and (0..len).all(i => opt(ct.left[i]) == opt((0..i).filter(j => par[j] == i).first()) and opt(ct.right[i]) == opt((i + 1..len).filter(j => par[j] == i).first()))
        var vw: List<(Int, Int)> = []
        for l in 1..len:
            for r in l + 1..len:
                if (l..r).all(j => a[j] < a[l - 1] and a[j] < a[r]): vw.push((l, r))
        let vg = [(v.l, v.r) for v in a.valleys()]
        let sp = a.spans()
        let sp_ok = (0..len).all(i => (sp[i].l, sp[i].r) == ((0..i).filter(j => a[j] <= a[i]).last().map(j => j + 1) ?? 0, (i + 1..len).filter(j => a[j] < a[i]).first() ?? len))
        let sm = rng.int(0, 1) == 1
        let ps = a.prev_smaller(strict: sm)
        let ns = a.next_smaller(strict: sm)
        let less = (x: Int, y: Int) => x < y or (not sm and x == y)
        let pn_ok = (0..len).all(i => opt(ps[i]) == opt((0..i).filter(j => less(a[j], a[i])).last()) and opt(ns[i]) == opt((i + 1..len).filter(j => less(a[j], a[i])).first()))
        let pq = a.prev_same()
        let nq = a.next_same()
        let same_ok = (0..len).all(i => opt(pq[i]) == opt((0..i).filter(j => a[j] == a[i]).last()) and opt(nq[i]) == opt((i + 1..len).filter(j => a[j] == a[i]).first()))
        let rs = [(rng.int(0, 5), rng.int(0, 3)) for _ in 0..rng.int(0, 5)]
        let rr = [l..l + w for (l, w) in rs]
        var ow: List<(Int, Int)> = []
        for (l, w) in rs:
            let r = l + w
            if not rs.any(o => o.0 <= l and r <= o.0 + o.1 and (o.0, o.0 + o.1) != (l, r)) and not ow.any(o => o == (l, r)): ow.push((l, r))
        ow.sort()
        let og = [(o.l, o.r) for o in rr.outermost()]
        let cap = rng.int(4, 9)
        let mx = a.maximal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state <= cap)
        let mx_ok = (0..len).all(r => (mx[r].l, mx[r].r) == ((0..=r).filter(l => (l..=r).map(j => a[j]).sum() <= cap).first()!, r + 1))
        let mn = a.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= cap)
        let mn_ok = (0..len).all(r => (if let w = mn[r]: (w.l, w.r) else: (-1, -1)) == (if let l = (0..=r).filter(l => (l..=r).map(j => a[j]).sum() >= cap).last(): (l, r + 1) else: (-1, -1)))
        if not ct_ok: why = "cartesian_tree {a} min_root {yes(min_root)}"
        elif vg != vw: why = "valleys {a}"
        elif not sp_ok: why = "spans {a}"
        elif not pn_ok: why = "prev/next_smaller {a} strict {yes(sm)}"
        elif not same_ok: why = "prev/next_same {a}"
        elif og != ow: why = "outermost {rs}"
        elif not mx_ok: why = "maximal {a} cap {cap}"
        elif not mn_ok: why = "minimal {a} cap {cap}"
        elif len > 0:
            let ring = a.ring
            let i = rng.int(0, len - 1)
            let d = rng.int(-9, 9)
            let kk = rng.int(1, len)
            let wins = ring.windows(kk).collect()
            let win_ok = wins.len == len and (0..len).all(s => wins[s].start == s and wins[s].len == kk and not wins[s].is_empty and wins[s].end == (s + kk - 1) % len and wins[s].each().collect() == [(s + j) % len for j in 0..kk] and wins[s].keys.collect() == [(s + j) % len for j in 0..kk] and wins[s].range.len == kk)
            let sum_at = (e: Int, l: Int) => (0..l).map(j => a[((e - j) % len + len) % len]).sum()
            let rmx = ring.maximal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state <= cap)
            let rmx_ok = (0..len).all(e => rmx[e].end == e and rmx[e].len == (1..=len).filter(l => sum_at(e, l) <= cap).last()!)
            let rmn = ring.minimal(state: track(0, (mut s, x) => s += x, (mut s, x) => s -= x), ok: w => w.state >= cap)
            let rmn_ok = (0..len).all(e => (if let w = rmn[e]: w.len else: -1) == ((1..=len).filter(l => sum_at(e, l) >= cap).first() ?? -1) and (if let w = rmn[e]: w.end == e else: true))
            if ring.step(i, d) != ((i + d) % len + len) % len or ring.next(i) != (i + 1) % len or ring.prev(i) != (i + len - 1) % len: why = "ring step {i} {d} of {len}"
            elif ring.from(i).collect() != [(i + j) % len for j in 0..len]: why = "ring from {i}"
            elif ring.span(i, kk).each().collect() != [(i + j) % len for j in 0..kk]: why = "ring span {i} {kk}"
            elif not win_ok: why = "ring windows {kk} of {a}"
            elif not rmx_ok: why = "ring maximal {a} cap {cap}"
            elif not rmn_ok: why = "ring minimal {a} cap {cap}"
    if why == "":
        let want_len = (0..pushed.len).map(i => lis_end(pushed, i, lis_strict)).max() ?? 0
        let rc = lis.reconstruct()
        let rc_ok = rc.len == want_len and (1..rc.len).all(j => rc[j - 1] < rc[j] and (pushed[rc[j - 1]] < pushed[rc[j]] or (not lis_strict and pushed[rc[j - 1]] == pushed[rc[j]])))
        if lis.len != want_len or lis.is_empty != (pushed.len == 0): why = "OnlineLis len of {pushed}"
        elif not rc_ok or lis.items.collect() != [(i, pushed[i]) for i in rc]: why = "OnlineLis reconstruct of {pushed}: {rc}"
        elif ms.items.collect() != st or ms.len != st.len or ms.is_empty != (st.len == 0): why = "MonoStack items"
        elif (if let t = ms.top: (t.index, t.val) else: (-1, -1)) != (if st.len > 0: st.last! else: (-1, -1)): why = "MonoStack top"
    if why != "":
        bad = "op {op}: {why}"
        break
print(if bad == "": "ok" else: bad)
