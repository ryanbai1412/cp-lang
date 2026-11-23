// expect: ok, output "6 6"
// rule: 08.5.3
// Generic over a capability trait on sequences instead of a higher-kinded type.
fn total<S: Seq<Int>>(xs: S) -> Int:
    var t = 0
    for x in xs: t += x
    t
print(total([1, 2, 3]), total(0..4))
