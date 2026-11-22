// expect: E0311 at 2
// rule: 06.4.1
@memo(array)
fn f(n: Int) -> Int = if n == 0: 0 else: f(n - 1)
print(f(3))
