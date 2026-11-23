// expect: ok, output "2 none 2 0 0 0 2"
use std.spec.{decrease, convexity_break, unimodality_break, monge_break, dnc_break}
let c = convexity_break([4, 1, 0, 1, 4])
let (mi, mj) = monge_break((i, j) => i * j, 3, 3)!
let (da, dc) = dnc_break((j, i) => 0 - (i - j) * (i - j), 4)!
print(decrease([1, 3, 2, 1])!, if c.is_some: "some" else: "none", unimodality_break([5, 3, 4])!, mi, mj, da, dc)
