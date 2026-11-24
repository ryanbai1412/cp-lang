// expect: ok, output "ok"
// input: "11 3000\n"
// model: LazyTreap
// A LazyTreap with a summary against a plain list: `prefix` folds the first `k` elements and
// `count_while` counts the run from the front that keeps a threshold, under range tags that add
// `d` to each element.
use std.algebra.{Action}
use std.lazy.{LazyTreap}
use std.rand.{Rng}

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var t = LazyTreap<Int, Int>(
    apply: (v, k, f) => v + f,
    compose: (g, f) => g + f,
    before: (x, y) => x > y,
    summary: Action<Int, Int>(apply: (d, s, m) => s + d * m),
)
var m: List<Int> = []
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 3)
    if op == 0 or m.len == 0:
        let x = rng.int(-20, 20)
        t.insert(x)
        var i = 0
        while i < m.len and m[i] >= x: i += 1
        m.insert(i, x)
    elif op == 1:
        let d = rng.int(-3, 3)
        t.apply_all(d)
        for i in 0..m.len: m[i] += d
    elif op == 2:
        let lo = rng.int(0, m.len)
        let d = rng.int(0, 3)
        // adding a non-negative amount to a prefix keeps the list falling
        t.apply_range(0, lo, d)
        for i in 0..lo: m[i] += d
    let q = rng.int(0, m.len)
    var want = 0
    for i in 0..q: want += m[i]
    if t.prefix(q) != want: bad = "prefix({q}) at op {k}"
    let v = rng.int(-25, 25)
    var c = 0
    while c < m.len and m[c] >= v: c += 1
    if t.count_while(x => x >= v) != c: bad = "count_while(>= {v}) at op {k}"
    if t.len != m.len: bad = "len at op {k}"
print(if bad == "": "ok" else: bad)
