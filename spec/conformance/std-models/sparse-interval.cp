// expect: ok, output "ok"
// input: "1 300\n"
// model: SparseIntervalMap
// model: SparseIntervalSet
// SparseIntervalMap<Int> and SparseIntervalSet over coordinates -20..20 against plain lists:
// seeded random assign (with its on_old pieces checked), update, rebuilds by new, add and remove,
// with get, run_at, each, each_rev, range, runs, show, contains, next_gap, covered and the runs
// compared after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.interval.{SparseIntervalMap, SparseIntervalSet}

fn runs_of(a: List<Int>) -> List<((Int, Int), Int)>:
    var out: List<((Int, Int), Int)> = []
    var lo = 0
    for i in 1..=a.len:
        if i == a.len or a[i] != a[lo]:
            out.push(((lo - 20, i - 20), a[lo]))
            lo = i
    out

fn set_runs(b: List<Bool>) -> List<(Int, Int)>:
    var out: List<(Int, Int)> = []
    var i = 0
    while i < b.len:
        if b[i]:
            var j = i
            while j < b.len and b[j]: j += 1
            out.push((i - 20, j - 20))
            i = j
        else:
            i += 1
    out

fn ends(r: Range<Int>) -> (Int, Int) = (r.start, r.end)

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var m = SparseIntervalMap(-20, 20, 0)
var a = List.filled(40, 0)
var s = SparseIntervalSet()
var b = List.filled(40, false)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 4)
    let lo = rng.int(-20, 20)
    let hi = rng.int(lo, 20)
    let x = rng.int(-25, 25)
    let v = rng.int(0, 3)
    if op == 0:
        var pieces: List<((Int, Int), Int)> = []
        m.assign(lo..hi, v, on_old: (rg, old) => pieces.push((ends(rg), old)))
        var pos = lo
        for (e, old) in pieces:
            if bad == "" and (e.0 != pos or e.1 <= e.0 or e.1 > hi):
                bad = "op {k}: assign {lo}..{hi} piece {e} at {pos}"
            for j in e.0..min(e.1, 20):
                if bad == "" and a[j + 20] != old: bad = "op {k}: assign {lo}..{hi} old {old} at {j}"
            pos = e.1
        if bad == "" and pos != hi: bad = "op {k}: assign {lo}..{hi} pieces end at {pos}"
        for j in lo..hi: a[j + 20] = v
    elif op == 1:
        m.update(lo..hi, y => (y + v) % 4)
        for j in lo..hi: a[j + 20] = (a[j + 20] + v) % 4
    elif op == 2:
        if rng.int(0, 5) == 0:
            m = SparseIntervalMap(-20, 20, v)
            a = List.filled(40, v)
        else:
            s.add(lo..hi)
            for j in lo..hi: b[j + 20] = true
    elif op == 3:
        s.remove(lo..hi)
        for j in lo..hi: b[j + 20] = false
    elif rng.int(0, 3) == 0:
        s = SparseIntervalSet()
        b = List.filled(40, false)
    else:
        let j = rng.int(-20, 19)
        s.add(j..j + 1)
        b[j + 20] = true
    if bad != "": break
    let i = rng.int(-20, 19)
    let want = runs_of(a)
    let got = [(ends(p.range), p.val) for p in m.each()]
    if got != want or m.runs != want.len:
        bad = "op {k}: map runs {got} want {want}"
    elif [(ends(p.range), p.val) for p in m.each_rev()] != [want[want.len - 1 - j] for j in 0..want.len]:
        bad = "op {k}: each_rev"
    elif m.get(i) != a[i + 20]:
        bad = "op {k}: get {i}"
    elif "{m}" != ", ".join(["{w.0.0}..{w.0.1}={w.1}" for w in want]):
        bad = "op {k}: show {m}"
    else:
        let w = [y for y in want if y.0.0 <= i and i < y.0.1][0]
        let clipped = [((max(y.0.0, lo), min(y.0.1, hi)), y.1) for y in want if y.0.0 < hi and lo < y.0.1 and lo < hi]
        if (ends(m.run_at(i).range), m.run_at(i).val) != w:
            bad = "op {k}: run_at {i} want {w}"
        elif [(ends(p.range), p.val) for p in m.range(lo..hi)] != clipped:
            bad = "op {k}: range {lo}..{hi}"
    if bad != "": break
    let sw = set_runs(b)
    var ng = x
    while ng >= -20 and ng < 20 and b[ng + 20]: ng += 1
    let inside = x >= -20 and x < 20 and b[x + 20]
    if [ends(y) for y in s.each()] != sw or s.runs != sw.len:
        bad = "op {k}: set runs {[ends(y) for y in s.each()]} want {sw}"
    elif s.covered != b.count(y => y):
        bad = "op {k}: covered"
    elif s.contains(x) != inside:
        bad = "op {k}: contains {x}"
    elif s.next_gap(x) != ng:
        bad = "op {k}: next_gap {x}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
