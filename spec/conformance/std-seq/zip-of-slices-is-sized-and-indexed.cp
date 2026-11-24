// expect: ok, output "3\n3 4\n2\n5 6"
// rule: 07.2.13
// A zip of two positional sequences is indexed: `.len` is the shorter's and `z[i]` reads both by position, with no copy. Only a zip with a walk-only side is walk-only.
use std.seq.{zip}
let xs = [1, 3, 5, 2, 4, 6]
let z = zip(xs[0..3], xs[3..6])
print(z.len)
print(z[1])
let w = zip(xs, [9, 8])
print(w.len)
print(z[2])
