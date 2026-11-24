// expect: ok, output "ok"
// input: "1 300\n"
// model: PersistentArray
// A PersistentArray against one plain list per version: seeded random set on a random old
// version, with get read from a random version after each op. Reads `seed n`.
use std.rand.{Rng}
use std.persistent.{PersistentArray, Version}
let seed0 = read<Int>()
let n = read<Int>()
domain A = 0..11
var rng = Rng(seed0)
let fill = rng.int(0, 1) == 0
let a0: List<Int> = [if fill: 7 else: rng.int(0, 50) for _ in 0..11]
var p = if fill: PersistentArray<A, Int>(fill: 7) else: PersistentArray(a: [A] i => a0[i.ord])
var vs: List<Version<PersistentArray<A, Int>>> = [p.base]
var bs: List<List<Int>> = [a0]
var bad = ""
for k in 0..n:
    let v = rng.int(0, vs.len - 1)
    let i = rng.int(0, 10)
    let x = rng.int(0, 50)
    vs.push(p.set(vs[v], A(i), x))
    var b = bs[v].clone()
    b[i] = x
    bs.push(b)
    let q = rng.int(0, vs.len - 1)
    let j = rng.int(0, 10)
    if p.get(vs[q], A(j)) != bs[q][j]: bad = "op {k}: get v{q} {j}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
