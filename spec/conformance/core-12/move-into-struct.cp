// expect: E0608 at 4
// rule: 12.4.12
struct S: xs: List<Int>
let a = [1]
let s = S(a)
print(a.len)
