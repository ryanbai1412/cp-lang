// 1957D A BIT of an Inequality: a_y must flip f(x,z)'s bit at a_y's top bit from 0 to 1.
use std.math.{bit_len}

@!multitest
let n: 1..=1e5 = read()
let a = read<[n]1..=1e9>()
var p: [n + 1]Int = 0
for i in 0..n:
    p[i + 1] = p[i] ^ a[i]
// ones[b, i]: how many of p[0..i] have bit b set
var ones: [30, n + 2]Int = 0
for b in 0..30, i in 0..=n:
    ones[b, i + 1] = ones[b, i] + Int(p[i].bit(b))
var ans = 0
for y in 1..=n:
    let h = bit_len(a[y - 1]) - 1
    let l1 = ones[h, y]
    let l0 = y - l1
    let r1 = ones[h, n + 1] - ones[h, y]
    let r0 = n + 1 - y - r1
    ans += l0 * r0 + l1 * r1
print(ans)
