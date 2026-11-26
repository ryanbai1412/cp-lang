// expect: ok, output "YES 4\nNO 2\nYES"
// A chain reads each operand once and stops at a false link (R01.5.3).
var calls = 0
fn side(x: Int) -> Int:
    calls += 1
    return x
print(yes(side(1) < side(2) <= side(2) < side(4)), calls)
calls = 0
print(yes(side(3) < side(1) < side(9)), calls)
print(yes('a' <= 'm' < 'z'))
