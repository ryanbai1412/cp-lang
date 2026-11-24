// expect: R0401
// rule: std.marks.Arena.free
use std.marks.{Arena, Id}
var a = Arena<Int>()
let i = a.alloc(5)
a.free(i)
a.free(i)  // <- here
