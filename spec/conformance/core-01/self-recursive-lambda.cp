// expect: ok, output "120"
// rule: 01.8.1
let f = (n: Int) -> Int => if n == 0: 1 else: n * f(n - 1)
print(f(5))
