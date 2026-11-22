// expect: ok, output "7"
// rule: 02.6.1
struct T: var x: Int
var t = T(1)
t.x = 7
print(t.x)
