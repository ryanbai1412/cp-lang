// expect: ok, output "5"
// rule: 01.3.4
struct Box<T>: v: T
let a = 1
let b = 2
let lt = a < b
let x = Box<Int>(5)
print(if lt: x.v else: 0)
