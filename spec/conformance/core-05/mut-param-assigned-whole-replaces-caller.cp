// expect: ok, output "9"
// rule: 05.4.3
fn reset(mut xs: List<Int>):
    xs = [9]
var a = [1, 2, 3]
reset(mut a)
print(a)
