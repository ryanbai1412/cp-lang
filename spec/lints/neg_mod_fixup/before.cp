let n: 1..=100 = read()
let a: Int = read()
let b: Int = read()
print((a - b + n) % n, (a % n + n) % n, ((a % 7) + 7) % 7)
