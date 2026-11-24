// expect: ok, output "2 3"
// Rng.sample from a program that loads nothing but std.rand: its body may not lean on
// std.seq's methods, which only a walk over a non-range loads.
use std.rand.{Rng}
var r = Rng(9)
print(r.sample(2, from: [5, 6, 7]).len, r.sample(3, from: [5, 6, 7]).len)
