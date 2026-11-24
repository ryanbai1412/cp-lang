// expect: E0608
// rule: 05.4.4
struct Holder(xs: List<Int>)
var a = [1, 2]
let h = Holder(a)
print(a.len) // <- here
