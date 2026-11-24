// expect: E0311
// rule: 06.4.1
@memo(array)
fn f(n: Int) -> Int = if n == 0: 0 else: f(n - 1) // <- here
print(f(3))
