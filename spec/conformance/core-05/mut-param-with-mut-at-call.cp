// expect: ok, output "1"
// rule: 05.4.1
fn grow(mut xs: List<Int>):
    xs.push(1)
var a = List<Int>()
grow(mut a)
print(a)
