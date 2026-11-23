let n: 1..=90 = read()
var memo: [0..=90]Int = -1  // expect: sentinel_table
fn fib(i: Int) -> Int:
    if i < 2:
        return i
    if memo[i] != -1:
        return memo[i]
    memo[i] = fib(i - 1) + fib(i - 2)
    return memo[i]
print(fib(n))
