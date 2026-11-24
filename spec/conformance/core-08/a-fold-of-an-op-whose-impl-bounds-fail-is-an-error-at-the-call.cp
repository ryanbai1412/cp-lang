// expect: E0606
// rule: 08.2.2
// `*`'s `Op` impl needs `T: Mul + One`, which `(Int, Int)` lacks, so each `fold(*)` call is refused where it is written; the error names `one`, not std's body.
let xs = [3, 1, 4]
fn a() = (List(zip(xs, xs))).fold(*) // <- here
fn b() = (List(zip(xs, xs))).fold(*)
