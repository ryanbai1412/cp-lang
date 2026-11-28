// expect: ok, output "3 9 2\n3 9 2\n6"
// rule: R04.8.11
// One generic fn over `[D]T` with `D: Discrete` runs at a counted domain (an array) and at a
// `[Int]Int` binding (a list, not a list for a counted array): its `len`, `keys` and `a[d]` are
// chosen at the instance.
fn sum_by<D: Discrete>(a: [D]Int) -> (Int, Int, Int):
    var s = 0
    for k in a.keys:
        s += a[k]
    (a.len, s, a[D.nth(1)])
domain V = 0..3
let av: [V]Int = [V] v => Int(v) * 2 + 2
let al: [Int]Int = [2, 2, 5]
let (n, s, m) = sum_by(av)
print(n, s - 3, m - 2)
let (n2, s2, m2) = sum_by(al)
print(n2, s2, m2)
let both: [V]Int = 2
print(both.len * both[V(0)])
