// expect: ok, output "ok"
// input: "1 300\n"
// model: abs
// model: floor
// model: ceil
// model: round
// model: trunc
// std.math's abs and F64 rounding against their definitions: each float is k + j/8, exact in
// binary, so floor is k and the others follow from j and the sign. Reads `seed n`; prints the
// first op that differs.
use std.rand.{Rng}

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for op in 0..n:
    let k = if rng.chance(0.1): rng.int(-(1 << 40), 1 << 40) else: rng.int(-1000, 1000)
    let j = rng.int(0, 7)
    let x = F64(k) + F64(j) / 8.0
    let up = if j > 0: k + 1 else: k
    let half = if j > 4 or (j == 4 and x > 0.0): k + 1 else: k
    let toward = if x >= 0.0: k else: up
    let mag = if k < 0: -k else: k
    let fmag = if x < 0.0: -x else: x
    if x.floor != k or floor(x) != k: bad = "op {op}: floor({x})"
    elif x.ceil != up or ceil(x) != up: bad = "op {op}: ceil({x})"
    elif x.round != half or round(x) != half: bad = "op {op}: round({x})"
    elif x.trunc != toward or trunc(x) != toward: bad = "op {op}: trunc({x})"
    elif k.abs != mag or abs(k) != mag or I128(k).abs != I128(mag): bad = "op {op}: abs({k})"
    elif x.abs != fmag or abs(x) != fmag: bad = "op {op}: abs({x})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
