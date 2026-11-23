// expect: ok, output "1"
// rule: 04.4.28
// A declared domain satisfies a `Domain` bound, so `buckets` takes an array whose values are a domain (bug-catch round 12, 108aae46b).
domain V = 0..3
domain W = 1..=3
let w: [V]W = [1, 1, 2]
let b = buckets(w)
print(1)
