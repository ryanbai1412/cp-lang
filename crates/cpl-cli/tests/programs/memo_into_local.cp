// `@memo(into: dp)` on a fn inside a fn, and inside a loop.
let n: Int = read()
fn solve() -> Int:
    @memo(into: dp)
    fn fib(i: 0..=n) -> Int = if i < 2: i else: fib(i - 1) + fib(i - 2)
    let r = fib(n)
    print(dp[n]!, dp[2]!, count(i for i in 0..=n if dp[i] != none))
    dp[n] = 7
    r + fib(n)
print(solve())
for round in 0..2:
    @memo(into: ways)
    fn f(i: 0..=n) -> Int:
        if i <= 1: return 1 + round
        f(i - 1) + f(i - 2)
    print(f(10), ways[3]!)
