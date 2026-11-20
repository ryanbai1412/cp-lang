// cp: 0.1 c217120+dirty
// CCO 2020 P1 A Game with Grundy: Grundy stands at an integer x in l..=r
// at height y. Friend i sits at (x_i, 0) and sees the points with
// |x - x_i| * v_i < h_i * y. For every k from 0 to n, print how many
// positions are seen by at most k friends.
use std.seq.{scan}

let n: 1..=1e5 = read()
let l: -1e9..=1e9 = read()
let r: l..=1e9 = read()
let y: 1..=1e9 = read()
let friends: [n](x: l..=r, v: 1..=100, h: 1..=100) = read()

var events: List<(pos: Int, delta: Int)> = [(l, 0), (r + 1, 0)]
for (x, v, h) in friends:
    let reach = (h * y - 1) / v             // the largest |x - x_i| still strictly inside the view
    let lo = max(l, x - reach)
    let hi = min(r, x + reach)
    if lo <= hi:
        events.push((lo, 1))
        events.push((hi + 1, -1))
events.sort()

var seen_by: [0..=n]Int = 0                 // positions watched by exactly this many friends
var cur = 0
var prev = l
for (pos, delta) in events:
    if pos > prev:
        seen_by[cur] += pos - prev
        prev = pos
    cur += delta
for total in scan(seen_by, +): print(total)
