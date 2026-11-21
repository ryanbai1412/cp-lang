const SHIFT = 10_000_000_000

@!multitest
let n: Int = read()
var sum_xor = 0
var x_xor = 0
for _ in 0..n:
    let (x0, y0): (Int, Int) = read()
    let x = x0 + SHIFT
    let y = y0 + SHIFT
    sum_xor ^= x + y
    x_xor ^= x
print(x_xor - SHIFT, (sum_xor - x_xor) - SHIFT)
