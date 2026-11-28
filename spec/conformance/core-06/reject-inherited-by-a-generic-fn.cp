// expect: E0303
// rule: 06.4.17
// engines: next
// has: "count(x => x)"
// A generic fn taking `count`'s default leaves `T = Bool` open, so it inherits the rejection: each call decides it with its own `T`.
fn all_of<T>(xs: List<T>) -> Int = xs.count()
print(all_of([1, 2, 3]))
print(all_of([true, false])) // <- here
