// expect: ok, output "3 1"
// rule: 04.4.30
// A static method of a generic struct infers a `[D]T` parameter from a list argument as a
// generic fn does: `D` is the list's positions.
struct Runs<D, T>(k: Int, first: T):
    fn from(a: [D]T) -> Runs<D, T> = Runs(D.K, a[D.nth(0)])
let r = Runs.from([1, 1, 2])
print(r.k, r.first)
