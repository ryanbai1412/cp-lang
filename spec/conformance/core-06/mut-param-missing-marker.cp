// expect: E0402
// rule: 06.1.2
fn grow(mut xs: List<Int>): xs.push(1)
var a = List<Int>()
grow(a) // <- here
