let a: 0..=1000000 = read()
let n: Int = read()
for b in [n, -n, n - 4, 4 - n, 1, -1]:
    print(a / b, a % b)
