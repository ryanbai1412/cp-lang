// A Fibonacci number wrong at 3, planted under a recursion for `cpl stress --explain`.
let n: 4..=9 = read()

fn fib(k: Int) -> Int:
    if k < 2: return k
    if k == 3: return 1
    return fib(k - 1) + fib(k - 2)

@alt(fib)
fn fib_slow(k: Int) -> Int:
    if k < 2: return k
    return fib_slow(k - 1) + fib_slow(k - 2)

print(fib(n))
