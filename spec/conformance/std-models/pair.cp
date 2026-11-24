// expect: ok, output "ok"
// input: "1 300\n"
// model: Pair
// model: symmetric
// Pair<V> against a list of (lo, hi) built by a double loop in walk order: seeded random
// new, at, other, ord and nth on random positions, and symmetric on a random per-pair table
// against the obvious square loop. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.pair.{Pair, symmetric}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..9
var rng = Rng(seed0)
var walk: List<(Int, Int)> = []
for h in 0..9:
    for l in 0..h: walk.push((l, h))
var bad = ""
if Pair<V>.count != walk.len: bad = "count {Pair<V>.count} want {walk.len}"
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 2)
    let u = rng.int(0, 8)
    let v = rng.int(0, 8)
    if op == 0:
        let p = Pair<V>.at(V(u), V(v))
        if u == v:
            if p != None: bad = "op {k}: at({u}, {u}) not None"
        else:
            let q = p!
            let (lo, hi) = (min(u, v), max(u, v))
            if Int(q.lo) != lo or Int(q.hi) != hi: bad = "op {k}: at({u}, {v}) = ({q.lo}, {q.hi})"
            elif q != Pair<V>.new(V(v), V(u)): bad = "op {k}: new({v}, {u}) differs from at"
            elif walk[q.ord] != (lo, hi): bad = "op {k}: ord({lo}, {hi}) = {q.ord}"
            elif Int(q.other(V(u))) != v or Int(q.other(V(v))) != u: bad = "op {k}: other"
    elif op == 1:
        let i = rng.int(0, walk.len - 1)
        let p = Pair<V>.nth(i)
        if (Int(p.lo), Int(p.hi)) != walk[i] or p.ord != i: bad = "op {k}: nth({i})"
    else:
        let d: [Pair<V>]Int = [Pair<V>] _ => rng.int(-50, 50)
        let diag = rng.int(-5, 5)
        let m = d.symmetric(diag:)
        for a in V:
            for b in V:
                let want = if a == b: diag else: d[Pair.new(a, b)]
                if bad == "" and m[a, b] != want: bad = "op {k}: symmetric[{a}, {b}] = {m[a, b]} want {want}"
print(if bad == "": "ok" else: bad)
