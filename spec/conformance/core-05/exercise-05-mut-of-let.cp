// expect: E0403 at 3
// rule: 05.3.1
fn grow(mut xs: List<Int>): xs.push(0)
let b = List<Int>()
grow(mut b)
