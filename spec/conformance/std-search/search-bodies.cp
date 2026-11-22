// expect: ok, output "YES\nYES\nYES\n123456 YES\n4 6 YES"
use std.search.{bsearch_real, ternary_max_real, ternary_min_real, ternary_max}
print(yes(abs(bsearch_real(0.0, 2.0, x => x * x >= 2.0)! - 1.4142135623730951) < 1e-9))
print(yes(abs(ternary_max_real(0.0, 4.0, x => -(x - 1.0) * (x - 1.0)) - 1.0) < 1e-6))
print(yes(abs(ternary_min_real(-3.0, 3.0, x => (x - 2.0) * (x - 2.0)) - 2.0) < 1e-6))
var calls = 0
let peak = ternary_max(0..1000000, i => { calls += 1; -(i - 123456) * (i - 123456) })!
print(peak, yes(calls <= 34))
domain P = 0..10
let r = P(2)..P(9)
print(Int(r.first_true(p => Int(p) >= 4)!), Int(r.last_true(p => Int(p) <= 6)!), yes(r.first_true(p => false) == None))
