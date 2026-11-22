// expect: E0608 at 4
// rule: 02.6.3
struct Bag: xs: List<Int>
let a = [1, 2]
let b = Bag(a)
print(a.len, b.xs.len)
