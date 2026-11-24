// expect: E0203
// rule: 07.1.1
fn evens(n: Int) -> yields Int:
    for i in 0..n: if i % 2 == 0: yield i
let g = evens(6)
print(g.next()) // <- here
