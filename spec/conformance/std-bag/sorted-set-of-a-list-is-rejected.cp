// expect: E0303
// `SortedSet(xs)` is an error, not an empty set; the contents go through `SortedSet.of(xs)`.
let a: List<Int> = [3, 1, 3]
let s = SortedSet(a) // <- here
