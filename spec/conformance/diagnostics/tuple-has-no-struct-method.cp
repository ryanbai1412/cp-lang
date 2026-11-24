// expect: E0203
// rule: 08.1.6
// A tuple has no `dot`: the free `dot` takes `Pt`s, so no call is offered.
use std.geometry.{Pt}
let p = (1, 2)
let q = (3, 4)
print(p.dot(q)) // <- here
