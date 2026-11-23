// expect: E0608 at 4
// rule: 05.4.4
struct Holder(xs: List<Int>)
var a = [1, 2]
let h = Holder(a)
print(a.len)
