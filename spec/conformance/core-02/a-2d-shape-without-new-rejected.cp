// expect: E0203
// rule: 02.11.4
domain R = 0..2
domain C = 0..3
struct Box<T>(v: T)
struct Box<R2, C2, T>(n: Int)
let a: [R, C]Int = 1
let b = Box(a) // <- here
print(b.n)
