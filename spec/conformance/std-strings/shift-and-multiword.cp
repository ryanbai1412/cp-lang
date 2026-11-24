// expect: ok, output "ok"
// shift_and against find_all on patterns of up to a few 64-bit words, over periodic texts
// with a little noise so long patterns match many times, and on a slice's own keys.
use std.rand.{Rng}
var rng = Rng(7)
var bad = ""
for k in 0..300:
    let blk = Str([Char(Int('a') + rng.int(0, 1)) for _ in 0..rng.int(1, 70)])
    let s = Str([if rng.chance(0.01): 'c' else: c for c in blk.repeat(rng.int(0, 8))])
    let lo = rng.int(0, s.len)
    let hi = rng.int(lo, min(s.len, lo + 200))
    let p = Str([s[j] for j in lo..hi])
    if s.shift_and(p).collect() != s.find_all(p).collect(): bad = "op {k}: {s} {p}"
    elif s.len > 2 and s[2..].shift_and(p).collect() != s[2..].find_all(p).collect(): bad = "op {k}: slice"
    if bad != "": break
print(if bad == "": "ok" else: bad)
