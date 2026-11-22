// expect: E0601 at 5
// rule: 05.8.1
struct Pair: left: List<Int>, right: List<Int>
fn both(mut x: List<Int>, mut y: List<Int>) = todo
var p = Pair(List(), List())
both(mut p.left, mut p.right)
both(mut p.left, mut p.left)
