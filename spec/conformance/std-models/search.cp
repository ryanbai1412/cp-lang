// expect: ok, output "ok"
// input: "1 300\n"
// model: bsearch
// model: ternary_max
// model: ternary_min
// model: peak
// model: valley
// model: parallel_bsearch
// model: gallop_first
// model: dinkelbach
// model: split_best
// The answer searches on random monotone, unimodal and ratio instances, each against a scan:
// the real bsearch against its own halving loop, the discrete ternaries against the first extreme,
// the real ones against the known optimum, parallel_bsearch against prefix sums, dinkelbach
// against the best ratio, and split_best against the sorted values, on `Int` ranges and on a
// cuts domain, whose positions have no `+ Int`. Reads `seed n`; prints the
// first op that differs.
use std.rand.{Rng}
use std.search.{gallop_first}

domain C = 0..6

fn near(a: F64, b: F64) -> Bool = abs(a - b) <= 1e-6

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let cs = [x for x in C.cuts]
var bad = ""
for k in 0..n:
    let lo = F64(rng.int(-20, 20)) / 4.0
    let hi = lo + F64(rng.int(0, 40)) / 4.0
    let t = lo - 1.0 + (hi - lo + 2.0) * rng.float()
    let steps = rng.int(0, 60)
    var (a, b) = (lo, hi)
    for _ in 0..steps:
        let m = (a + b) / 2.0
        if m >= t: b = m else: a = m
    let want: ?F64 = if hi >= t: Some(b) else: None
    if (lo..hi).bsearch(x => x >= t, steps:) != want: bad = "op {k}: ({lo}..{hi}).bsearch(>= {t}, steps: {steps})"
    let c = lo + (hi - lo) * rng.float()
    if not near((lo..hi).peak(by: x => -(x - c) * (x - c)), c): bad = "op {k}: peak of a parabola at {c}"
    elif not near((lo..hi).valley(by: x => (x - c) * (x - c)), c): bad = "op {k}: valley of a parabola at {c}"
    elif not near((lo..hi).peak(by: x => -abs(x - c)), c): bad = "op {k}: peak at {c}"
    elif not near((lo..hi).valley(by: x => abs(x - c)), c): bad = "op {k}: valley at {c}"
    if bad != "": break

    let len = rng.int(0, 12)
    let top = if len == 0: 0 else: rng.int(0, len - 1)
    let flat = if len == 0: 0 else: rng.int(1, len - top)
    var f: List<Int> = []
    for i in 0..len:
        if i < top: f.push(if i == 0: rng.int(-5, 5) else: f[i - 1] + rng.int(1, 3))
        elif i < top + flat: f.push(if i == 0: rng.int(-5, 5) else: f[i - 1] + (if i == top: rng.int(1, 3) else: 0))
        else: f.push(f[i - 1] - rng.int(1, 3))
    let off = rng.int(-5, 5)
    var first_max: ?Int = None
    for i in 0..len:
        if first_max == None and f[i] == f.max()!: first_max = Some(i + off)
    if ternary_max(off..off + len, i => f[i - off]) != first_max: bad = "op {k}: ternary_max {f}"
    elif ternary_min(off..off + len, i => -f[i - off]) != first_max: bad = "op {k}: ternary_min {f}"
    if bad != "": break

    let from = rng.int(-10, 10)
    let at = from + rng.int(-3, 40)
    if gallop_first(from, i => i >= at) != Some(max(from, at)): bad = "op {k}: gallop_first({from}, >= {at})"
    let m = rng.int(0, 8)
    let w = [rng.int(0, 3) for _ in 0..m]
    let need = [rng.int(0, 10) for _ in 0..rng.int(0, 5)]
    var want_t: List<?Int> = []
    for q in 0..need.len:
        var s = 0
        var hit: ?Int = None
        for i in 0..m:
            s += w[i]
            if hit == None and s >= need[q]: hit = Some(i + off)
        want_t.push(hit)
    var cnt = 0
    if parallel_bsearch(need.len, off..off + m, reset: () => cnt = 0, apply: i => cnt += w[i - off], ok: q => cnt >= need[q]) != want_t:
        bad = "op {k}: parallel_bsearch w {w} need {need}"
    if bad != "": break

    let ch = [(num: F64(rng.int(0, 9)), den: F64(rng.int(1, 5))) for _ in 0..rng.int(1, 5)]
    let best = [x.num / x.den for x in ch].max()!
    let got = dinkelbach(0.0, solve: l => ch[(0..ch.len).max(by: i => ch[i].num - l * ch[i].den)!])
    if not near(got, best): bad = "op {k}: dinkelbach {got} want {best}"
    let groups = [(l..l + rng.int(0, len - l)) for l in [rng.int(0, len) for _ in 0..rng.int(0, 3)]]
    let vals = [f[i] for g in groups, i in g].sorted(by: x => desc(x))
    if split_best(groups, range: g => g, best: (_, r) => r.map(i => (f[i], i)).max()!).collect() != vals: bad = "op {k}: split_best"
    let fc = [rng.int(-9, 9) for _ in cs]
    let cg: List<Range<C.cuts>> = [cs[l]..cs[l + rng.int(0, cs.len - 1 - l)] for l in [rng.int(0, cs.len - 1) for _ in 0..rng.int(0, 3)]]
    let cv = [fc[x.ord] for g in cg, x in g].sorted(by: x => desc(x))
    if split_best<Range<C.cuts>, C.cuts, Int>(cg, range: g => g, best: (_, r) => r.map(x => (fc[x.ord], x)).max()!).collect() != cv:
        bad = "op {k}: split_best on a cuts domain"
    if bad != "": break
print(if bad == "": "ok" else: bad)
