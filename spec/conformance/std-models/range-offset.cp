// expect: ok, output "ok"
// input: "1 300\n"
// model: offset
// model: ahead
// model: back
// A position's offset, ahead and back against stepping its ordinal by hand: seeded random
// domain positions and steps, some far past either end. Reads `seed n`; prints the first op
// that differs.
use std.rand.{Rng}

domain V = 0..9
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
fn step(o: Int) -> ?V = if 0 <= o < 9: Some(V.nth(o)) else: None
for op in 0..n:
    if bad != "": break
    let k = if rng.int(0, 9) == 0: rng.int(-(1 << 61), 1 << 61) else: rng.int(-12, 12)
    let v = V.nth(rng.int(0, 8))
    if v.offset(k) != step(v.ord + k): bad = "op {op}: offset"
    elif v.ahead(k) != step(v.ord + k): bad = "op {op}: ahead"
    elif v.back(k) != step(v.ord - k): bad = "op {op}: back"
print(if bad == "": "ok" else: bad)
