// expect: E0402 at 4
// rule: 05.3.5
fn grow(mut xs: List<Int>): xs.push(1)
var a = List<Int>()
grow(mut a)                 // fine
grow(a)                     // error: grow takes `mut xs`; write grow(mut a)
let b = List<Int>()
grow(mut b)                 // error: b is a let; declare it var
