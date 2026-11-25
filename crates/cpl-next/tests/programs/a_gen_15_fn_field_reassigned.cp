// expect: ok, output "7"
// rule: decision 1
// A function value stored in a struct field, reassigned, then called; the set has two members.
struct Op:
    var f: (Int) -> Int
var o = Op((x: Int) => x + 1)
let k = 5
o.f = (x: Int) => x + k
print(o.f(2))
