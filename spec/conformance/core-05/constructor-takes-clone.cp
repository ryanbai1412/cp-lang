// expect: ok, output "2 2"
// rule: 05.4.4
struct Holder(xs: List<Int>)
var a = [1, 2]
let h = Holder(a.clone())
print(a.len, h.xs.len)
