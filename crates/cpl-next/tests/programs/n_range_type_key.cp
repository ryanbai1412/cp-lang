// expect: ok, output "10 1\n1\n10\n20"
// rule: R04.3.4
// An array keyed by a range type argument takes an `Int` key the way an array over an
// anonymous range does.
fn mk<D: Domain>() -> [D]Int = [D] d => d.ord * 10
var a = mk<2..=4>()
a[2] += 1
print(a[3], a[2])
for x in a: print(x)
