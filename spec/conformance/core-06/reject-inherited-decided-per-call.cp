// expect: ok, output "3 4 1"
// rule: 06.4.17
// A generic fn's inherited rejection refuses only the calls whose types make it hold: here none does.
fn all_of<T>(xs: List<T>) -> Int = xs.count()
fn twice<U>(xs: List<U>) -> Int = 2 * all_of(xs)
print(all_of([1, 2, 3]), twice(["a", "b"]), all_of([[true]]))
