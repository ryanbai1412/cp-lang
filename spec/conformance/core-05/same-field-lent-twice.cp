// expect: E0601 at 4
// rule: 05.8.1
struct S(left: Int, right: Int)
fn both(mut a: Int, mut b: Int): a += b
var s = S(1, 2)
both(mut s.left, mut s.left)
