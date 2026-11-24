// expect: ok, output "1 3 3\n1 2 3 4 5 7\n9 9"
// A fn's own generic `D` bound to an `Int`-keyed array's keys keeps `[D]T` that array, so a
// `mut` argument writes back in place; only a struct's `D = Int` makes `[D]T` a list (R07.3.5).
fn bump<D>(mut d: [D]Int, k: Int): d[k] += 1
fn total<D>(d: [D]Int) -> Int:
    var s = 0
    for x in d: s += x
    s
var xs: [0..3]Int = [1, 2, 3]
bump(mut xs, 1)
print(xs)
var ys: [0..6]Int = [1, 2, 3, 4, 5, 6]
bump(mut ys, 5)
print(ys)
print(total(xs) + 2, total([4, 5]))
