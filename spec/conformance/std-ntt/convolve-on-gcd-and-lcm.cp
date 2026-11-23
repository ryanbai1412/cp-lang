// expect: ok, output "4 0 7 1 1\n4 0 1 3 3"
// `on: gcd` and `on: lcm` index from 1 and keep `max(a.len, b.len)` cells; `lcm` drops the rest.
use std.ntt.*
let g = convolve([0, 1, 1, 1], [0, 1, 1, 1], on: gcd)
print(g.len, g[0], g[1], g[2], g[3])
let l = convolve([0, 1, 1, 1], [0, 1, 1, 1], on: lcm)
print(l.len, l[0], l[1], l[2], l[3])
