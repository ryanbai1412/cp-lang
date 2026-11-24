// expect: ok, output "YES"
// model: gen_fields
// Against brute force: every draw is in each field's range, and 600 draws
// reach each of the 6 field combinations ((5/6)^600 odds of missing one).
use std.derive.gen_fields

struct P(a: 0..=2, b: 5..=6)
impl Random for P:
    fn gen() -> P = gen_fields<P>()

var seen = [false, false, false, false, false, false]
var ok = true
for _ in 0..600:
    let p = gen<P>()
    ok = ok and p.a >= 0 and p.a <= 2 and p.b >= 5 and p.b <= 6
    seen[p.a * 2 + (p.b - 5)] = true
for s in seen: ok = ok and s
print(yes(ok))
