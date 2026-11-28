// expect: E0303
// rule: 06.4.17
// engines: next
// has: "count(x => x)"
// An inherited rejection is inherited again by a generic caller, however deep.
fn all_of<T>(xs: List<T>) -> Int = xs.count()
fn twice<U>(xs: List<U>) -> Int = 2 * all_of(xs)
print(twice(['a', 'b']))
print(twice([false])) // <- here
