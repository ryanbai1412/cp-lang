// cp: 0.1 17af32c+dirty
domain I = 0..10

@memo(into: dp)
fn fib(i: I) -> Int:
    if i.idx < 2: return i.idx
    fib(I.nth(i.idx - 1)) + fib(I.nth(i.idx - 2))

dp[I.nth(5)] = 100
print(fib(I.last))
print(count(dp[i] != none for i in I), dp[I.nth(7)] ?? -1)

domain J = 0..3
@memo(into: grid)
fn paths(i: J, j: J) -> Int:
    if let a = i.prev:
        if let b = j.prev:
            return paths(a, j) + paths(i, b)
    1
print(paths(J.last, J.last), grid[J.last, J.first] ?? -1, grid[J.first, J.last] ?? -1)
