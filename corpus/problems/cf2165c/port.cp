// Source: Codeforces 2165 C (Binary Wine), https://codeforces.com/contest/2165/problem/C
// limits: 2000 ms, 512 MB
// Problem: n values a_i < 2^30; one coin raises an a_i by 1. Each of q independent queries
//   gives c < 2^30: print the fewest coins so some b with 0 <= b_i <= a_i has XOR c.
//   Multitest, sum of n <= 5e5, sum of q <= 5e4.
// Solution: go from the high bit down. If more values have bit b than c needs, that value can
//   take any lower bits: done. If c needs bit b and no value has it, raise the value with the
//   largest low part to the next multiple of 2^b. At most 30 values are raised, always among
//   the 30 largest, so keep only those: O(n log n + q 30^2).

use std.seq.{argmax}

const BITS = 30

@!multitest
let (n, q): (1..=500_000, 1..=50_000) = read()
let a: [0..n]0..=2 ** 30 - 1 = read()
let top = a.sorted(by: x => desc(x))[..min(n, BITS)].to_list()

for _ in 0..q:
    var buf = top
    let c: 0..=2 ** 30 - 1 = read()
    var ans = 0
    for b in (0..BITS).rev:
        let want = Int(c.bit(b))
        let cnt = buf.count(x => x.bit(b))
        if cnt > want: break
        if cnt < want:
            let mask = (1 << b) - 1
            let best_i = buf.argmax(by: x => x & mask)!
            let add = (1 << b) - (buf[best_i] & mask)
            ans += add
            buf[best_i] += add
    print(ans)
