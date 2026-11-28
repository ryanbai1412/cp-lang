// expect: ok, output "ok"
// input: "1 300\n"
// model: try_nth
// Against brute force: seeded random ordinals `k`, near and past both ends of `3..=9` and of
// `Int`, each read back through `D.try_nth(k)`, `v.next(k)`, `v.offset(k)` and a cut's `l`/`r`.
use std.rand.{Rng}

domain V = 3..=9

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
for step in 0..n:
    if bad != "": break
    let k = rng.int(-9, 16)
    let inside = 0 <= k and k < 7
    let got = V.try_nth(k)
    if inside != (got != None) or (inside and Int(got!) != k + 3): bad = "try_nth {step}"
    if Int.try_nth(k) != Some(k): bad = "int try_nth {step}"
    let v = V(rng.int(3, 9))
    let o = Int(v) - 3 + k
    let want = if 0 <= o and o < 7: Some(o + 3) else: None
    let nx = v.next(k)
    if (nx != None) != (want != None) or (want != None and Int(nx!) != want!): bad = "next {step}"
    let of = v.offset(k)
    if (of != None) != (want != None) or (want != None and Int(of!) != want!): bad = "offset {step}"
    let c = rng.int(0, 7)
    let l = V.cuts(c).l
    let r = V.cuts(c).r
    if (l != None) != (c > 0) or (c > 0 and Int(l!) != c + 2): bad = "cut l {step}"
    if (r != None) != (c < 7) or (c < 7 and Int(r!) != c + 3): bad = "cut r {step}"
print(if bad == "": "ok" else: bad)
