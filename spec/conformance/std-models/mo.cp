// expect: ok, output "ok"
// input: "1 300\n"
// model: mo_order
// model: mo_steps
// model: mo
// model: mo_order_hilbert
// model: mo_rollback
// model: mo_with_updates
use std.rand.{Rng}
use std.math.{isqrt}
use std.mo.{mo, mo_order, mo_order_hilbert, mo_rollback, mo_steps, mo_with_updates}

domain P = 0..12
const N = 12

fn rg(l: Int, r: Int) -> Range<P> = if r == N: P(l).. else: P(l)..P(r)

fn lo_hi(q: Range<P>) -> (Int, Int) = if q.is_empty: (0, 0) else: (q.first!.ord, q.last!.ord + 1)

// The Hilbert curve walked forward (Wikipedia's d2xy), so a point's key is looked up, not computed.
fn hilbert_keys(side: Int) -> List<Int>:
    var key = List.filled(side * side, 0)
    for d in 0..side * side:
        var (x, y, t, s) = (0, 0, d, 1)
        while s < side:
            let rx = (t / 2) % 2
            let ry = (t ^ rx) % 2
            if ry == 0:
                if rx == 1:
                    x = s - 1 - x
                    y = s - 1 - y
                (x, y) = (y, x)
            x += s * rx
            y += s * ry
            t /= 4
            s *= 2
        key[x * side + y] = d
    key

fn sorted_by(keys: List<Int>) -> List<Int> = [p.1 for p in [(keys[i], i) for i in 0..keys.len].sorted()]

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
for k in 0..n:
    if bad != "": break
    let a = [rng.int(0, 4) for _ in 0..N]
    var qs: List<Range<P>> = []
    var ends: List<(Int, Int)> = []
    for _ in 0..rng.int(0, 7):
        let l = rng.int(0, N - 1)
        let r = rng.int(l, N)
        qs.push(rg(l, r))
        ends.push((l, r))
    let want_sum = [(l..r).map(i => a[i]).sum() for (l, r) in ends]
    let want_distinct = [(l..r).map(i => a[i]).collect().distinct().len for (l, r) in ends]
    let op = rng.int(0, 5)
    if op == 0:
        let got = mo_order(qs)
        var want: List<Int> = []
        if not ends.is_empty:
            let b = max(1, ends.map(e => e.1).max()! / isqrt(ends.len))
            want = sorted_by([(l / b) * 100 + (if l / b % 2 == 0: r else: 99 - r) for (l, r) in ends])
        if got != want: bad = "op {k}: mo_order({ends}) = {got} want {want}"
    elif op == 1:
        let l1 = rng.int(0, N - 1)
        let f = rg(l1, rng.int(l1, N))
        let t0 = rng.int(0, N - 1)
        let t = rg(t0, rng.int(t0, N))
        let (fl, fr) = f.ends
        let (tl, tr) = t.ends
        var have = [fl <= i and i < fr for i in 0..N]
        var want: List<(Int, Int)> = []
        for i in (tl..fl).rev: want.push((i, 1))
        for i in fr..tr: want.push((i, 1))
        for i in min(fl, tl)..tl: want.push((i, 0))
        for i in (tr..max(fr, tr)).rev: want.push((i, 0))
        let got = mo_steps(f, t)
        for s in got:
            let i = s.pos.ord
            if have[i] == s.entering: bad = "op {k}: mo_steps({f}, {t}) steps on {i} twice"
            have[i] = s.entering
        if bad == "" and have != [tl <= i and i < tr for i in 0..N]: bad = "op {k}: mo_steps({f}, {t}) ends wrong"
        if bad == "" and [(s.pos.ord, if s.entering: 1 else: 0) for s in got] != want: bad = "op {k}: mo_steps({f}, {t}) = {[(s.pos.ord, if s.entering: 1 else: 0) for s in got]} want {want}"
    elif op == 2:
        var cnt = List.filled(5, 0)
        var inw = List.filled(N, false)
        var s = 0
        var d = 0
        let got = mo(qs,
            add: p =>
                inw[p.ord] = true
                s += a[p.ord]
                if cnt[a[p.ord]] == 0: d += 1
                cnt[a[p.ord]] += 1,
            remove: p =>
                inw[p.ord] = false
                s -= a[p.ord]
                cnt[a[p.ord]] -= 1
                if cnt[a[p.ord]] == 0: d -= 1,
            answer: qi => (s, d, inw == [ends[qi].0 <= i and i < ends[qi].1 for i in 0..N]))
        if got.map(x => x.0).collect() != want_sum: bad = "op {k}: mo sums over {ends} = {got.map(x => x.0).collect()} want {want_sum}"
        elif got.map(x => x.1).collect() != want_distinct: bad = "op {k}: mo distinct over {ends}"
        elif got.any(x => not x.2): bad = "op {k}: mo window is not the query at answer time over {ends}"
    elif op == 3:
        let got = mo_order_hilbert(qs)
        var want: List<Int> = []
        if not ends.is_empty:
            let lo = ends.map(e => e.0).min()!
            let span = ends.map(e => e.1).max()! - lo
            var side = 1
            while side <= span: side *= 2
            let key = hilbert_keys(side)
            want = sorted_by([key[(l - lo) * side + (r - lo)] for (l, r) in ends])
        if got != want: bad = "op {k}: mo_order_hilbert({ends}) = {got} want {want}"
    elif op == 4:
        var cnt = List.filled(N, 0)
        var resets = 0
        let got = mo_rollback(qs,
            mark: () => cnt.clone(),
            add: p => cnt[p.ord] += 1,
            answer: qi => (cnt == [if ends[qi].0 <= i and i < ends[qi].1: 1 else: 0 for i in 0..N], (0..N).map(i => cnt[i] * a[i]).sum()),
            undo: m => cnt = m,
            reset: () =>
                resets += 1
                cnt = List.filled(N, 0))
        if got.any(x => not x.0): bad = "op {k}: mo_rollback state is not the query at answer time over {ends}"
        elif got.map(x => x.1).collect() != want_sum: bad = "op {k}: mo_rollback sums over {ends}"
    else:
        var b = a.clone()
        let ups = [(pos: rng.int(0, N - 1), by: rng.int(-3, 3)) for _ in 0..rng.int(0, 5)]
        let tq = [(range: qs[i], time: rng.int(0, ups.len)) for i in 0..qs.len]
        var s = 0
        let got = mo_with_updates(tq, ups,
            add: p => s += b[p.ord],
            remove: p => s -= b[p.ord],
            apply_update: (u, w) =>
                b[u.pos] += u.by
                if w.contains(P(u.pos)): s += u.by,
            undo_update: (u, w) =>
                b[u.pos] -= u.by
                if w.contains(P(u.pos)): s -= u.by,
            answer: _ => s)
        var want: List<Int> = []
        for i in 0..tq.len:
            var c = a.clone()
            for j in 0..tq[i].time: c[ups[j].pos] += ups[j].by
            want.push((ends[i].0..ends[i].1).map(x => c[x]).sum())
        if got != want: bad = "op {k}: mo_with_updates({ends}, {ups}) = {got} want {want}"
print(if bad == "": "ok" else: bad)
