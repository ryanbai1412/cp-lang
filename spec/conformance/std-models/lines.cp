// expect: ok, output "ok"
// input: "1 300\n"
// model: Line
// model: LiChao
// model: LineContainer
// model: MonotoneHull
// LiChao (min and max, whole lines and segments), LineContainer (min and max, any slope order)
// and MonotoneHull (sorted slopes, tagged) against a List of the lines added, scanned at every
// queried x: seeded random adds, queries and clears. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.dp.{Line, LiChao, LineContainer, MonotoneHull}

fn best(ls: List<Line>, x: Int, lo: Bool) -> ?Int:
    var b: ?Int = None
    for l in ls:
        let v = l.k * x + l.m
        if b == None or (lo and v < b!) or (not lo and v > b!): b = Some(v)
    b

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let xmax = 20
var lc = [LiChao(min, over: 0..=xmax), LiChao(max, over: 0..=xmax)]
var lcs: List<List<(Line, Int, Int)>> = [[], []]
var cs = [LineContainer(min), LineContainer(max)]
var csl: List<List<Line>> = [[], []]
var hs = [MonotoneHull<Int>(min), MonotoneHull<Int>(max)]
var hsl: List<List<(Line, Int)>> = [[], []]
var hk = [100, -100]
var hx = [-30, -30]
var tag = 0
var bad = ""
for k in 0..n:
    let s = rng.int(0, 1)
    let lo = s == 0
    let op = rng.int(0, 8)
    let l = Line(rng.int(-5, 5), rng.int(-20, 20))
    let x = rng.int(0, xmax)
    if l.at(x) != l.k * x + l.m: bad = "op {k}: Line.at"
    if op <= 1:
        if op == 0:
            lc[s].add(l)
            lcs[s].push((l, 0, xmax))
        else:
            let a = rng.int(0, xmax)
            let b = rng.int(a, xmax)
            lc[s].add(l, on: a..=b)
            lcs[s].push((l, a, b))
    elif op == 2:
        cs[s].add(l)
        csl[s].push(l)
    elif op == 3 or op == 4:
        let kk = hk[s] + (if lo: -rng.int(0, 2) else: rng.int(0, 2))
        let hl = Line(kk, l.m)
        hk[s] = kk
        hs[s].add(hl, tag: tag)
        hsl[s].push((hl, tag))
        tag += 1
    elif op == 5:
        let q = hx[s] + rng.int(0, 6)
        hx[s] = q
        let want = best([e.0 for e in hsl[s]], q, lo)
        let got = hs[s].best_next(q)
        if want == None:
            if got != None: bad = "op {k}: best_next on empty"
        elif got == None: bad = "op {k}: best_next {q} None"
        else:
            let g = got!
            if g.val != want! or not any(e.1 == g.tag and e.0.at(q) == want! for e in hsl[s]):
                bad = "op {k}: best_next {q} = ({g.val}, {g.tag}) want {want!}"
    elif op == 6:
        let q = hx[s] + rng.int(0, 6)
        hx[s] = q
        if hs[s].at_next(q) != best([e.0 for e in hsl[s]], q, lo): bad = "op {k}: at_next {q}"
    elif op == 7 and rng.int(0, 3) == 0:
        let w = rng.int(0, 2)
        if w == 0:
            lc[s].clear()
            lcs[s] = []
        elif w == 1:
            cs[s] = if lo: LineContainer(min) else: LineContainer(max)
            csl[s] = []
        else:
            hs[s].clear()
            hsl[s] = []
            hk[s] = if lo: 100 else: -100
            hx[s] = -30
    if bad != "": break
    for q in 0..=xmax:
        let want = best([e.0 for e in lcs[s] if e.1 <= q and q <= e.2], q, lo)
        if lc[s].at(q) != want: bad = "op {k}: LiChao.at {q}"
        elif let ln = lc[s].line_at(q):
            if Some(ln.at(q)) != want or not any(e.0 == ln and e.1 <= q and q <= e.2 for e in lcs[s]): bad = "op {k}: LiChao.line_at {q}"
        elif want != None: bad = "op {k}: LiChao.line_at {q} None"
        if bad != "": break
    let kept = cs[s].each().collect()
    if bad == "" and (cs[s].len != kept.len or cs[s].is_empty != (kept.len == 0) or kept.len > csl[s].len):
        bad = "op {k}: LineContainer.len"
    let hkept = hs[s].items.collect()
    if bad == "" and (hs[s].len != hkept.len or hs[s].is_empty != (hkept.len == 0) or hkept.len > hsl[s].len):
        bad = "op {k}: MonotoneHull.len"
    for q in -30..=30:
        if bad != "": break
        let want = best(csl[s], q, lo)
        if cs[s].at(q) != want or best(kept, q, lo) != want: bad = "op {k}: LineContainer.at {q}"
        elif let ln = cs[s].line_at(q):
            if Some(ln.at(q)) != want or not any(e == ln for e in csl[s]): bad = "op {k}: LineContainer.line_at {q}"
        elif want != None: bad = "op {k}: LineContainer.line_at {q} None"
        let hw = best([e.0 for e in hsl[s]], q, lo)
        if bad == "" and (hs[s].at(q) != hw or best([e.0 for e in hkept], q, lo) != hw): bad = "op {k}: MonotoneHull.at {q}"
        if bad == "" and not all(any(e == h for e in hsl[s]) for h in hkept): bad = "op {k}: MonotoneHull.items"
    if bad != "": break
print(if bad == "": "ok" else: bad)
