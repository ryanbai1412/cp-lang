// expect: ok, output "5 4 2"
// rule: 02.10.3
// A range type's end is a `const` or a constant expression of them, in a fn's parameter and
// in a struct's field alike: check works each out once.
const N = 3 + 2
const M = N * 2
struct Pick:
    k: 0..M
fn last(x: 0..=N) -> Int = x
fn back(x: 1..N) -> Int = N - x
let p = Pick(k: 5)
print(last(5), p.k - 1, back(3))
