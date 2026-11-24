use std.window.{track}
let n: Int = read()
let ws = [i % 5 for i in 0..n]
var s = 0
for r in 0..16:
    s += ws.minimal(state: track(0, (mut t, x) => t += x, (mut t, x) => t -= x), ok: w => w.state >= 6).len // timed: seq.minimal
    s += ws.ring.minimal(state: track(0, (mut t, x) => t += x, (mut t, x) => t -= x), ok: w => w.state >= 6).len // timed: seq.Ring.minimal
print(s)
