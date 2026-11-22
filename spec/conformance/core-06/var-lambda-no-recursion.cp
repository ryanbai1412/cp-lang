// expect: E0406 at 1
// rule: 06.2.3
var f = (n: Int) -> Int => if n == 0: 0 else: f(n - 1)
