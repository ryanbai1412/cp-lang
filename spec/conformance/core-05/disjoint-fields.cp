// expect: ok, output "2 0"
// rule: 05.8.1
struct S:
    var left: List<Int>
    var right: List<Int>
fn mv(mut a: List<Int>, mut b: List<Int>): a.push(b.pop()!)
var s = S(left: [1], right: [2])
mv(mut s.left, mut s.right)
print(s.left.len, s.right.len)
