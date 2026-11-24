// Source: Codeforces 2165 C (Binary Wine), https://codeforces.com/contest/2165/problem/C
// limits: 2000 ms, 512 MB
// Problem: n values a_i < 2^30; one coin raises an a_i by 1. Each of q independent queries
//   gives c < 2^30: print the fewest coins so some b with 0 <= b_i <= a_i has XOR c.
//   Multitest, sum of n <= 5e5, sum of q <= 5e4.
// Solution: go from the high bit down. If more values have bit b than c needs, that value can
//   take any lower bits: done. If c needs bit b and no value has it, raise the value with the
//   largest low part to the next multiple of 2^b. At most 30 values are raised, always among
//   the 30 largest, so keep only those: O(n + q 30^2).
// Bounds: n: 1..=500_000; q: 1..=50_000; a: 0..=2 ** 30 - 1; c: 0..2 ** 30

const BITS = 30

@!multitest
let (n, q): (1..=500_000, 1..=50_000) = read()
let a: [0..n]0..=2 ** 30 - 1 = read()
let top = a.max_k(BITS)

for _ in 0..q:
    let c: 0..=2 ** 30 - 1 = read()
    var raised = 0  // bit i: top[i] went up to a multiple of 2^b, so no bit below b is left
    var ans = 0
    for b in (0..BITS).rev:
        let want = Int(c.bit(b))
        let cnt = count(not raised.bit(i) and top[i].bit(b) for i in 0..top.len)
        if cnt > want: break
        if cnt < want:
            let mask = (1 << b) - 1
            let low = (i: Int) => if raised.bit(i): 0 else: top[i] & mask
            let best_i = (0..top.len).argmax(by: low)!
            ans += (1 << b) - low(best_i)
            raised |= 1 << best_i
    print(ans)
