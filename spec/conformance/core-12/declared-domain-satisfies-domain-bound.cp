// expect: ok, output "1"
// rule: 
// A declared domain satisfies a `Domain` bound, so `buckets` takes an array whose values are a domain (bug-catch round 12, 108aae46b).
use std.seq.{buckets}
domain V = 0..3
domain W = 1..=3
let w: [V]W = [1, 1, 2]
let b = buckets(w)
print(1)
