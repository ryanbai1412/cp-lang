// expect: ok, output "23416728348467685"
// rule: 06.4.1
@memo(map)
fn fib(n: Int) -> Int = if n < 2: n else: fib(n - 1) + fib(n - 2)
print(fib(80))
