// expect: ok, output "120 55 6 10"
// rule: R06.2.3, R06.2.1
// A `let`-bound lambda may call itself; its captures go with it into each call.
let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)
fn fib(k: Int) -> Int:
    let base = 2
    let f = (n: Int) -> Int => if n < base: n else: f(n - 1) + f(n - 2)
    f(k)
fn walk(xs: List<Int>) -> Int:
    var total = 0
    let go = (i: Int) -> () =>
        if i < xs.len:
            total += xs[i]
            go(i + 1)
    go(0)
    total
fn sum_all() -> Int:
    let xs = [1, 2, 3, 4]
    let sum = (i: Int) -> Int => if i == xs.len: 0 else: xs[i] + sum(i + 1)
    sum(0)
print(fact(5), fib(10), walk([1, 2, 3]), sum_all())
