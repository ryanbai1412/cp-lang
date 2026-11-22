// expect: ok, output "YES NO Impossible ans=YES\n8 10 1000000007\nYES"
use std.io.{yes}
use std.search.{gallop_first}
print(yes(true), yes(false), yes(false, answers: ("Possible", "Impossible")), "ans={yes(true)}")
print(gallop_first(0, i => i * i >= 50), gallop_first(10, i => i >= 3), gallop_first(-5, i => i >= 1_000_000_007))
var ok = true
for t in 0..200:
    for f in -3..3:
        if gallop_first(f, i => i >= t) != Some(max(f, t)): ok = false
print(ok)
