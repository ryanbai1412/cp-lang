// expect: E0302
// `List(5)` is not C++'s `vector(5)` of five zeros: `List(xs)` copies a sequence, so an `Int` is an error (write `[0 for _ in 0..5]` or `[5]`).
let a: List<Int> = List(5) // <- here
