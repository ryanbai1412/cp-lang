// expect: R0401
// rule: std.marks.Arena.free
use std.marks.{Arena, Id}
var a = Arena<Int>()
a.free(Id<Int>(0))  // <- here
