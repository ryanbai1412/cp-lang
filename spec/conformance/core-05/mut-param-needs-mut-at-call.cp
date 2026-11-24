// expect: E0402
// rule: 05.4.1
fn grow(mut xs: List<Int>):
    xs.push(1)
var a = List<Int>()
grow(a) // <- here
