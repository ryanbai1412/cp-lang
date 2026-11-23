let n: 1..=100 = read()
let a: Int = read()
let b: Int = read()
print((a - b + n) % 7, (a + 1) % n, (a - b + 2 * n) % n)
