// expect: ok, output "ok"
// input: "1 300\n"
// model: IntervalMap
// model: IntervalSet
// IntervalMap<D, Int> against a List<Int> and IntervalSet<D> against a List<Bool>: seeded random
// assign (with its on_old pieces checked), assign_point, set, rebuilds by new and from, add and
// remove, with get, run_at, each, each_rev, range, first_where, runs, contains, contains_range,
// next_covered, next_gap and covered compared after each op. Reads `seed n`; prints the first op
// that differs.
use std.rand.{Rng}
use std.range.{ends}
use std.interval.{IntervalMap, IntervalSet}
domain D = 0..20

fn runs_of(a: List<Int>) -> List<((Int, Int), Int)>:
    var out: List<((Int, Int), Int)> = []
    var lo = 0
    for i in 1..=a.len:
        if i == a.len or a[i] != a[lo]:
            out.push(((lo, i), a[lo]))
            lo = i
    out

fn set_runs(b: List<Bool>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    var i = 0
    while i < b.len:
        if b[i]:
            var j = i
            while j < b.len and b[j]: j += 1
            out.push((i, j))
            i = j
        else:
            i += 1
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var m = IntervalMap<D, Int>(0)
var a = List.filled(20, 0)
var s = IntervalSet<D>()
var b = List.filled(20, false)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let lo = rng.int(0, 20)
    let hi = rng.int(lo, 20)
    let r = D.cuts.nth(lo)..D.cuts.nth(hi)
    let i = rng.int(0, 19)
    let v = rng.int(0, 3)
    if op == 0:
        var pieces: List<((Int, Int), Int)> = []
        m.assign(r, v, on_old: (rg, old) => pieces.push((rg.ends, old)))
        var pos = lo
        for (e, old) in pieces:
            if bad == "" and (e.0 != pos or e.1 <= e.0 or e.1 > hi):
                bad = "op {k}: assign {lo}..{hi} piece {e} at {pos}"
            for j in e.0..min(e.1, 20):
                if bad == "" and a[j] != old: bad = "op {k}: assign {lo}..{hi} old {old} at {j}"
            pos = e.1
        if bad == "" and pos != hi: bad = "op {k}: assign {lo}..{hi} pieces end at {pos}"
        for j in lo..hi: a[j] = v
    elif op == 1:
        if m.assign_point(D(i), v) != a[i]: bad = "op {k}: assign_point {i} old"
        a[i] = v
    elif op == 2:
        m.set(D(i), v)
        a[i] = v
    elif op == 3:
        if rng.int(0, 3) == 0:
            m = IntervalMap<D, Int>(v)
            a = List.filled(20, v)
        else:
            a = [rng.int(0, 2) for _ in 0..20]
            m = IntervalMap.from([D] p => a[p.ord])
    elif op == 4:
        s.add(r)
        for j in lo..hi: b[j] = true
    elif op == 5:
        s.remove(r)
        for j in lo..hi: b[j] = false
    else:
        if rng.int(0, 5) == 0: s = IntervalSet<D>()
        if rng.int(0, 5) == 0: b = List.filled(20, false)
        for j in 0..20:
            if b[j]: s.add(D(j).l..D(j).r)
            else: s.remove(D(j).l..D(j).r)
    if bad != "": break
    let want = runs_of(a)
    let got = [(p.range.ends, p.val) for p in m.each()]
    if got != want or m.runs != want.len:
        bad = "op {k}: map runs {got} want {want}"
    elif [(p.range.ends, p.val) for p in m.each_rev()] != [want[want.len - 1 - j] for j in 0..want.len]:
        bad = "op {k}: each_rev"
    elif m.get(D(i)) != a[i]:
        bad = "op {k}: get {i}"
    else:
        let (ri, vi) = (m.run_at(D(i)).range.ends, m.run_at(D(i)).val)
        let w = [x for x in want if x.0.0 <= i and i < x.0.1][0]
        let clipped = [((max(x.0.0, lo), min(x.0.1, hi)), x.1) for x in want if x.0.0 < hi and lo < x.0.1 and lo < hi]
        var fw: ?Int = None
        for j in lo..20:
            if fw == None and a[j] == v: fw = Some(j)
        if (ri, vi) != w:
            bad = "op {k}: run_at {i} = {ri} {vi} want {w}"
        elif [(p.range.ends, p.val) for p in m.range(r)] != clipped:
            bad = "op {k}: range {lo}..{hi}"
        elif m.first_where(D.cuts.nth(lo), x => x == v).map(p => p.ord) != fw:
            bad = "op {k}: first_where {lo} == {v}"
    if bad != "": break
    let sw = set_runs(b)
    var nc: ?Int = None
    var ng: ?Int = None
    for j in i..20:
        if nc == None and b[j]: nc = Some(j)
        if ng == None and not b[j]: ng = Some(j)
    let rw = [x for x in sw if x.0 <= i and i < x.1]
    if [x.ends for x in s.each()] != sw or s.runs != sw.len:
        bad = "op {k}: set runs {[x.ends for x in s.each()]} want {sw}"
    elif s.covered != b.count(x => x):
        bad = "op {k}: covered"
    elif s.contains(D(i)) != b[i]:
        bad = "op {k}: contains {i}"
    elif s.contains_range(r) != (lo..hi).all(j => b[j]):
        bad = "op {k}: contains_range {lo}..{hi}"
    elif s.run_at(D(i)).map(x => x.ends) != rw.first:
        bad = "op {k}: set run_at {i}"
    elif s.next_covered(D(i)).map(p => p.ord) != nc or s.next_gap(D(i)).map(p => p.ord) != ng:
        bad = "op {k}: next_covered/next_gap {i}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
