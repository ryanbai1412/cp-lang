let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)

fn run_fib(k: Int) -> Int:
    var calls = 0
    let fib = (n: Int) -> Int =>
        calls += 1
        if n <= 1: n else: fib(n - 1) + fib(n - 2)
    let ans = fib(k)
    return ans * 100 + calls

let gcd: (Int, Int) -> Int = (a, b) => if b == 0: a else: gcd(b, a % b)

print(fact(6), run_fib(7), gcd(48, 18))
