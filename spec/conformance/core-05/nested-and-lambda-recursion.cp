// expect: ok, output "32"
// rule: 05.10.5 06.2.3
fn outer() -> Int:
    fn fact(n: Int) -> Int = if n == 0: 1 else: n * fact(n - 1)
    let fib = (n: Int) -> Int => if n < 2: n else: fib(n - 1) + fib(n - 2)
    fact(4) + fib(6)
print(outer())
