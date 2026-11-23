let n: 1..=90 = read()
var memo: [0..=90]?Int = none
fn fib(i: Int) -> Int:
    if i < 2:
        return i
    if let v = memo[i]:
        return v
    let v = fib(i - 1) + fib(i - 2)
    memo[i] = v
    v
print(fib(n))
