// expect: ok, output "YES"
// rule: std.math.Ratio
// Every Ratio op on small fractions agrees with plain integer arithmetic: comparisons by
// cross-multiplying, `+ - * /` and `.p`, `.q` against lowest terms by a gcd.
use std.math.{Ratio}
fn lowest(p: Int, q: Int) -> (Int, Int):
    let g = if q < 0: -gcd(p, q) else: gcd(p, q)
    (p / g, q / g)
var ok = true
for a in -4..=4:
    for b in -4..=4:
        if b == 0: continue
        let x = Ratio(a, b)
        let s = if b < 0: -1 else: 1
        ok = ok and (x.p, x.q) == lowest(a, b) and x.floor == a * s / (b * s)
        for c in -3..=3:
            for d in 1..=3:
                let y = Ratio(c, d)
                let (l, r) = (a * s * d, c * b * s)
                ok = ok and yes(x < y) == yes(l < r) and yes(x == y) == yes(l == r)
                ok = ok and
                    (x + y).p == lowest(a * d + c * b, b * d).0 and
                    (x * y).q == lowest(a * c, b * d).1
                ok = ok and yes(Set([x, y]).len == 1) == yes(l == r)
                if c != 0: ok = ok and ((x / y).p, (x / y).q) == lowest(a * d, b * c)
print(yes(ok))
