fn ceil_log3(n: Int) -> Int = (0..).find(k => 3 ** k >= n)!

let n: 1..=1000000000 = read()
print(ceil_log3(n))
