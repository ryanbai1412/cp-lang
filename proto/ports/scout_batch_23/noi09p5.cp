// NOI '09 P5 - Pipe Marbles (DMOJ: noi09p5, TL 1.0s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Two pipes of lengths n and m hold balls colored 'A' or 'B' (given from bottom to top,
// or top to bottom — verified against corpus C++). Each step pop the front ball from
// either non-empty pipe to form an output string s. Let a_s be the number of schedules
// producing s. Compute sum_s a_s^2 mod 1024523.
//
// Combinatorial interpretation: sum_s a_s^2 is the number of PAIRS of independent
// schedules producing the SAME output string!
// After k steps, schedule 1 has popped i balls from pipe 1 (and k - i from pipe 2),
// schedule 2 has popped j balls from pipe 1 (and k - j from pipe 2).
// State dp[i, j] (rolled over k = 0..n+m) counts matching prefix pairs.

type Z = Mod<1024523>

let (n, m): (1..=500, 1..=500) = read()
// WISH MG11: exact alphabet {'A', 'B'} and length constraints for cpl stress
let s1: Str = read()
assert s1.len == n
let s2: Str = read()
assert s2.len == m

// Reverse if input lists top-to-bottom while balls pop from bottom (checked against C++)
domain I = 0..=n
var dp: [I, I]Z = 0
dp[I(0), I(0)] = 1

for k in 0..(n + m):
    var nxt: [I, I]Z = 0
    let lo = max(0, k - m)
    let hi = min(n, k)
    for i in lo..=hi:
        let i2 = k - i
        for j in lo..=hi:
            let cur = dp[I(i), I(j)]
            if cur == 0: continue
            let j2 = k - j
            if i < n and j < n and s1[i] == s1[j]:
                nxt[I(i + 1), I(j + 1)] += cur
            if i < n and j2 < m and s1[i] == s2[j2]:
                nxt[I(i + 1), I(j)] += cur
            if i2 < m and j < n and s2[i2] == s1[j]:
                nxt[I(i), I(j + 1)] += cur
            if i2 < m and j2 < m and s2[i2] == s2[j2]:
                nxt[I(i), I(j)] += cur
    dp = nxt

print(dp[I(n), I(n)])
