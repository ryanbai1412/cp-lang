// expect: E0303 at 4
// `Interner(xs)` is an error, not an empty interner.
use std.marks.{Interner}
let a: List<Int> = [3, 1, 3]
let t = Interner(a)
