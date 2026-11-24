// expect: E0704
// rule: 08.4.1
// `adjacency`'s own bound `R: (u: V, v: V, ..)` rejects a walk of non-records.
use std.graph.{adjacency}
let a = adjacency([1, 2]) // <- here
