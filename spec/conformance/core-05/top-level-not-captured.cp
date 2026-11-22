// expect: ok, output "5"
// rule: 05.10.1
var total = 0
fn add(x: Int): total += x
add(2)
add(3)
print(total)
