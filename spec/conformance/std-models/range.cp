// expect: ok, output "ok"
// input: "1 300\n"
// model: l
// model: r
// model: mid
// model: halve
// model: splits
// model: clip
// model: ends
// model: first_true
// model: last_true
// model: bsearch
// model: RingRange
// Range helpers on random `lo..hi` against loops over the range's elements: cuts, halves, splits,
// clip, the three binary searches on a threshold, and a random ring span against its listed keys.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let lo = rng.int(0, 12)
    let hi = rng.int(lo, 12)
    let r = lo..hi
    let xs = [x for x in r]
    let len = hi - lo
    let t = rng.int(-1, 14)
    if r.l.ord != lo or r.r.ord != hi or r.ends != (lo, hi): bad = "op {k}: l/r/ends of {lo}..{hi}"
    elif r.mid.ord != lo + len / 2: bad = "op {k}: mid of {lo}..{hi}"
    elif r.halve() != (if len <= 1: None else: Some((lo..lo + len / 2, lo + len / 2..hi))):
        bad = "op {k}: halve {lo}..{hi}"
    elif r.splits.collect() != [(lo..lo + j, lo + j..hi) for j in 1..len]: bad = "op {k}: splits {lo}..{hi}"
    if bad != "": break
    let a = rng.int(0, 12)
    let b = rng.int(a, 12)
    let both = [x for x in xs if a <= x < b]
    let c = r.clip(a..b)
    if [x for x in c] != both or c.len != both.len: bad = "op {k}: {lo}..{hi} clip {a}..{b}"
    if bad != "": break
    var first: ?Int = None
    var last: ?Int = None
    var below = 0
    var upto = 0
    for x in xs:
        if x >= t and first == None: first = Some(x)
        if x <= t: last = Some(x)
        if x < t: below += 1
        if x <= t: upto += 1
    if r.first_true(x => x >= t) != first: bad = "op {k}: first_true {lo}..{hi} >= {t}"
    elif r.last_true(x => x <= t) != last: bad = "op {k}: last_true {lo}..{hi} <= {t}"
    elif r.bsearch(x => x >= t).ord != lo + below: bad = "op {k}: bsearch {lo}..{hi} >= {t}"
    elif r.bsearch(x => x <= t, last: true).ord != lo + upto: bad = "op {k}: bsearch last {lo}..{hi} <= {t}"
    if bad != "": break
    let m = rng.int(1, 8)
    let ring = List.filled(m, 0).ring
    let s = rng.int(0, m - 1)
    let sl = rng.int(0, m)
    let span = ring.span(s, sl)
    let keys = [(s + j) % m for j in 0..sl]
    let q = rng.int(0, m - 1)
    if span.len != sl or span.is_empty != (sl == 0) or span.start != s: bad = "op {k}: ring span({s}, {sl}) of {m}"
    elif span.keys.collect() != keys or span.each().collect() != keys: bad = "op {k}: ring keys span({s}, {sl}) of {m}"
    elif sl > 0 and span.end != keys[sl - 1]: bad = "op {k}: ring end span({s}, {sl}) of {m}"
    elif span.contains(q) != keys.contains(q): bad = "op {k}: ring contains {q} span({s}, {sl}) of {m}"
    elif "{span}" != " ".join(["{x}" for x in keys]): bad = "op {k}: ring show span({s}, {sl}) of {m}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
