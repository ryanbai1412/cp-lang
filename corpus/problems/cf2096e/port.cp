// Source: Codeforces 2096E (Wonderful Teddy Bears), https://codeforces.com/contest/2096/problem/E
// limits: 2000 ms, 250 MB
// Problem: a row of n bears, black (B) or pink (P). One move sorts three consecutive bears so the
//   B's come first. Print the fewest moves that put every B before every P. Multitest, sum of
//   n <= 2e5.
// Solution: a move removes at most 2 inversions (P before B); removing 2 moves one bear by 2 and
//   keeps every bear's index parity, removing 1 is an adjacent swap that changes the number of P's
//   on odd indices by one. With D = |P's on odd indices - odd indices among the final P slots| the
//   answer is D + (inv - D) / 2 = (inv + D) / 2. One scan from the right counts the inversions and
//   the P's on odd indices (both counted from the right end). O(n).
// Bounds: n: 1..=200_000

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()
var cnt_b = 0
var cnt_p_odd = 0
var inv = 0
for (i, c) in s.rev.items:
    if c == 'P':
        inv += cnt_b
        cnt_p_odd += (n - 1 - i) % 2
    else:
        cnt_b += 1
let parity_fix = abs(cnt_p_odd - (n - cnt_b) / 2)
print((inv + parity_fix) / 2)
