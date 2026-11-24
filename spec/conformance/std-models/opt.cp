// expect: ok, output "ok"
// input: "1 300\n"
// model: is_some_and
// model: is_none_or
// model: and_then
// The optional methods against `if let`: seeded random `?Int` values and thresholds, with each
// method compared to the plain match it abbreviates. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let v: ?Int = if rng.chance(0.3): None else: Some(rng.int(-9, 9))
    let t = rng.int(-9, 9)
    var some_and = false
    var none_or = true
    var then: ?Int = None
    if let x = v:
        some_and = x > t
        none_or = x > t
        if x % 2 == 0: then = Some(x * t)
    if v.is_some_and(x => x > t) != some_and: bad = "op {k}: is_some_and({v ?? 99}, > {t})"
    elif v.is_none_or(x => x > t) != none_or: bad = "op {k}: is_none_or({v ?? 99}, > {t})"
    elif v.and_then(x => if x % 2 == 0: Some(x * t) else: None) != then: bad = "op {k}: and_then({v ?? 99})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
