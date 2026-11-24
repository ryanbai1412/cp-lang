// expect: ok, output "ok"
// input: "1 300\n"
// model: prod
// model: sum
// model: mean
// model: min
// model: max
// model: minmax
// model: count
// model: any
// model: all
// model: is_sorted
// model: argmin
// model: argmax
// model: max_count
// model: min_count
// model: majority
// model: reduce
// model: first
// model: last
// model: find
// model: rfind
// model: first_some
// model: tally
// model: group
// model: min_k
// model: max_k
// The std folds over a List against plain loops: each op draws a short random list of
// (value, position) pairs and checks every fold, extreme (with both tie rules), search, count
// and grouping. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let len = rng.int(0, 7)
    let a = [rng.int(-3, 3) for _ in 0..len]
    let b = [(a[i], i) for i in 0..len]
    let t = rng.int(-3, 3)
    let kk = rng.int(0, 8)
    var (s, p, pos, neg) = (0, 1, 0, 0)
    var (lo, hi, lo_l, hi_l) = (-1, -1, -1, -1)
    var sorted = true
    var (fi, li) = (-1, -1)
    for i in 0..len:
        s += a[i]
        p *= a[i]
        if a[i] > 0: pos += 1
        if a[i] < 0: neg += 1
        if lo < 0 or a[i] < a[lo]: lo = i
        if hi < 0 or a[i] > a[hi]: hi = i
        if lo_l < 0 or a[i] <= a[lo_l]: lo_l = i
        if hi_l < 0 or a[i] >= a[hi_l]: hi_l = i
        if i > 0 and a[i - 1] > a[i]: sorted = false
        if a[i] > t and fi < 0: fi = i
        if a[i] > t: li = i
    var why = ""
    if a.sum() != s: why = "sum"
    elif a.sum(of: x => x * x) != sum([x * x for x in a]): why = "sum of"
    elif a.prod() != p: why = "prod"
    elif a.count(x => x > 0) != pos or a.count() != len: why = "count"
    elif a.any(x => x < 0) != (neg > 0) or a.all(x => x > 0) != (pos == len): why = "any/all"
    elif a.is_sorted() != sorted: why = "is_sorted"
    elif b.is_sorted(by: q => -q.1) != (len <= 1): why = "is_sorted by"
    elif len == 0:
        if a.min() != None or a.max() != None or a.minmax() != None or a.mean() != None: why = "empty extremes"
        elif a.argmin() != None or a.argmax() != None or a.max_count() != None or a.min_count() != None: why = "empty arg"
        elif a.reduce((x, y) => x + y) != None or a.first != None or a.last != None: why = "empty reduce"
    else:
        let m = a.mean()!
        if m - F64(s) / F64(len) > 1e-9 or F64(s) / F64(len) - m > 1e-9: why = "mean"
        elif b.min(by: .0) != Some(b[lo]) or b.min(by: .0, ties: Ties.Last) != Some(b[lo_l]): why = "min"
        elif b.max(by: .0) != Some(b[hi]) or b.max(by: .0, ties: Ties.Last) != Some(b[hi_l]): why = "max"
        elif b.minmax(by: .0) != Some((b[lo], b[hi])) or a.minmax() != Some((a[lo], a[hi])): why = "minmax"
        elif a.argmin() != Some(lo) or a.argmin(ties: Ties.Last) != Some(lo_l): why = "argmin"
        elif a.argmax() != Some(hi) or a.argmax(ties: Ties.Last) != Some(hi_l): why = "argmax"
        elif a.argmax(by: x => -x) != Some(lo): why = "argmax by"
        elif b.max_count(by: .0) != Some((val: b[hi], count: a.count(x => x == a[hi]))): why = "max_count"
        elif b.min_count(by: .0) != Some((val: b[lo], count: a.count(x => x == a[lo]))): why = "min_count"
        elif a.reduce((x, y) => x * 3 - y) != a.drop(1).fold((x, y) => x * 3 - y, a[0]): why = "reduce"
        elif a.first != Some(a[0]) or a.last != Some(a[len - 1]): why = "first/last"
    if why == "":
        var maj: ?(val: Int, count: Int) = None
        for x in a:
            let c = a.count(y => y == x)
            if 2 * c > len: maj = Some((val: x, count: c))
        let tl = a.tally()
        let g = b.group(by: q => q.0 * q.0, of: q => q.1)
        if a.majority() != maj: why = "majority"
        elif a.find(x => x > t) != (if fi < 0: None else: Some(a[fi])): why = "find"
        elif a.rfind(x => x > t) != (if li < 0: None else: Some(a[li])): why = "rfind"
        elif a.first_some(x => if x > t: Some(x * 10) else: None) != (if fi < 0: None else: Some(a[fi] * 10)): why = "first_some"
        elif any(tl[v] != a.count(x => x == v) for v in -4..5): why = "tally"
        elif g.len != [q.0 * q.0 for q in b].distinct().len: why = "group len"
        elif any(g[q.0 * q.0] != [r.1 for r in b if r.0 * r.0 == q.0 * q.0] for q in b): why = "group"
        elif b.min_k(kk, by: .0).sorted(by: q => q.1) != b.sorted(by: q => q.0).take(kk).collect().sorted(by: q => q.1): why = "min_k"
        elif b.max_k(kk, by: .0).sorted(by: q => q.1) != b.sorted(by: q => (-q.0, q.1)).take(kk).collect().sorted(by: q => q.1): why = "max_k"
    if why != "":
        bad = "op {k}: {why} on {a} (t {t}, k {kk})"
        break
print(if bad == "": "ok" else: bad)
