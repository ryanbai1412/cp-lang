// cp: 0.1 c217120+dirty
// CCO 2020 P1 A Game with Grundy: Grundy stands at an integer x in l..=r
// at height y. Friend i sits at (x_i, 0) and sees the points with
// |x - x_i| * v_i < h_i * y. For every k from 0 to n, print how many
// positions are seen by at most k friends.
use std.seq.{running_sum}
use std.steps.{Steps}

let n: 1..=1e5 = read()
let l: -1e9..=1e9 = read()
let r: l..=1e9 = read()
let y: 1..=1e9 = read()
let friends: [n](x: l..=r, v: 1..=100, h: 1..=100) = read()

var watchers = Steps.new()
for (x, v, h) in friends:
    let reach = (h * y - 1) / v             // the largest |x - x_i| still strictly inside the view
    watchers.add(x - reach ..= x + reach, 1)

var seen_by: [0..=n]Int = 0                 // positions watched by exactly this many friends
for (span, count) in watchers.runs(l..=r): seen_by[count] += span.len
for total in running_sum(seen_by): print(total)
