// expect: E0403
// rule: 06.1.2
fn grow(mut xs: List<Int>): xs.push(1)
let b = List<Int>()
grow(mut b) // <- here
