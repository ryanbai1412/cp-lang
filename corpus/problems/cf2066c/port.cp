// Source: Codeforces 2066C (Bitwise Slides), https://codeforces.com/contest/2066/problem/C
// limits: 2000 ms, 250 MB
// Problem: n <= 2e5 values a_i <= 1e9; P = Q = R = 0. Xor each a_i in order into exactly one of
//   P, Q, R so that after every step the three are not pairwise distinct. Count the valid
//   choice sequences mod 1e9 + 7. Multitest, sum n <= 2e5.
// Solution: P ^ Q ^ R is the prefix xor, so a valid state is (x, x, prefix). ways[x] counts
//   them; a step only changes ways[previous prefix] = 3 * ways[prev] + 2 * ways[new prefix].
//   Answer: the sum of ways. A HashMap, O(n) expected.
@!multitest
let a: [0..read(1..=2e5)]1..=1e9 = read()

// after each step two of P, Q, R are equal (say x) and the third is the prefix xor;
// ways[x] counts those states, and only ways[previous prefix] changes per step
var ways: HashMap<Int, Mod<1e9 + 7>> = [0: 1]
var pre = 0
for x in a:
    ways[pre] = 3 * (ways.get(pre) ?? 0) + 2 * (ways.get(pre ^ x) ?? 0)
    pre ^= x
print(ways.values.sum())
