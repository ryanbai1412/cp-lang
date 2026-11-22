// expect: ok, output "2 2"
// rule: 02.6.3
struct Bag: xs: List<Int>
let a = [1, 2]
let b = Bag(a.clone())
print(a.len, b.xs.len)
