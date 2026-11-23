// Source: Codeforces 2237D (Fullmetal Bitchemist), https://codeforces.com/contest/2237/problem/D
// Problem: a binary string is beautiful if replacing two equal adjacent characters by one
//   opposite character, repeatedly, can bring it to length 1. Count the beautiful non-empty
//   substrings of s. Multitest, sum of n <= 1e6.
// Solution: weigh '0' as 1 and '1' as 2; a move keeps the weight mod 3, and a substring is
//   beautiful iff its weight is nonzero mod 3 and it is not alternating of length >= 2 (no move).
//   For each end i, count earlier prefixes with another residue (cnt[r] of prefix sums mod 3),
//   then drop the odd alternating suffixes of length >= 3 inside the alternating run ending at i,
//   (run - 1) / 2 of them (even ones already weigh 0). O(n).

@!multitest
let n: 1..=1_000_000 = read()
let s: [0..n]Char = read()
var ans = 0
var cnt: [0..3]Int = [1, 0, 0]
var pref = 0
var cur_len = 0
for (i, c) in s.items:
    pref = (pref + (c - '0') + 1) % 3
    cur_len = if i == 0 or c == s[i - 1]: 1 else: cur_len + 1
    ans += i + 1 - cnt[pref] - (cur_len - 1) / 2
    cnt[pref] += 1
print(ans)
