// expect: ok, output "2880067194370816120 2880067194370816120"
// rule: 13.3.4
@memo(array)
fn fib(n: 0..=90) -> Int = if n < 2: n else: fib(n - 1) + fib(n - 2)
@memo(map)
fn fib2(n: Int) -> Int = if n < 2: n else: fib2(n - 1) + fib2(n - 2)
print(fib(90), fib2(90))
