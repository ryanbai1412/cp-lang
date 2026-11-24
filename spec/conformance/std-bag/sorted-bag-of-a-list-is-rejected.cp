// expect: E0303
// `SortedBag(xs)` is an error, not an empty bag; the contents go through `SortedBag.of(xs)`.
let a: List<Int> = [3, 1, 3]
let b = SortedBag(a) // <- here
