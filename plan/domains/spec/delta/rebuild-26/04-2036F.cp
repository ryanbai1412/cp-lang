// 2036F XORificator 3000: xor of [l, r] minus (by xor) the run k + t * 2^i inside it.
fn xor_upto(x: Int) -> Int = [x, 1, x + 1, 0][x % 4]
fn xor_range(lo: Int, hi: Int) -> Int = xor_upto(hi) ^ xor_upto(lo - 1)

@!multitest
let (l, r, i, k): (1..=1e18, 1..=1e18, 0..=30, Int) = read()
let step = 1 << i
let first = (l - k + step - 1) / step
let last = (r - k) / step
var bad = 0
if first <= last:
    bad = xor_range(first, last) << i
    if (last - first + 1) % 2 == 1: bad ^= k
print(xor_range(l, r) ^ bad)
