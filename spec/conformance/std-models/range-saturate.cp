// expect: ok, output "ok"
// input: "1 300\n"
// model: shift_saturating
// A saturating shift of a position or a cut against clamping its ordinal by hand: seeded random
// domain sizes, starts and steps, some far past either end. Reads `seed n`; prints the first op
// that differs.
use std.rand.{Rng}

domain V = 0..9
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for op in 0..n:
    if bad != "": break
    let k = if rng.int(0, 9) == 0: rng.int(-(1 << 61), 1 << 61) else: rng.int(-12, 12)
    let v = V.nth(rng.int(0, 8))
    let c = V.cuts.nth(rng.int(0, 9))
    if v.shift_saturating(k) != V.nth(min(max(v.ord + k, 0), 8)): bad = "op {op}: position"
    elif c.shift_saturating(k) != V.cuts.nth(min(max(c.ord + k, 0), 9)): bad = "op {op}: cut"
print(if bad == "": "ok" else: bad)
