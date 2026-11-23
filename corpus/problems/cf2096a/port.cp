// Source: Codeforces 2096A (Wonderful Sticks), https://codeforces.com/contest/2096/problem/A
// Problem: arrange sticks of distinct lengths 1..n (n <= 100) in a row so that stick i + 1 is
//   shorter than every earlier stick if s_i = '<' and longer than every one if s_i = '>';
//   print any such arrangement. Multitest (t <= 500).
// Solution: fill from the right, keeping the unused lengths as an interval [lo, hi]: all of
//   them go earlier, so '>' takes hi and '<' takes lo. The value left over goes first. O(n).

@!multitest
let n: 2..=100 = read()
let s: [0..n - 1]Char = read()
var lo = 1
var hi = n
var ans: [0..n]Int = 0
for i in (0..n - 1).rev:
    if s[i] == '>':
        ans[i + 1] = hi
        hi -= 1
    else:
        ans[i + 1] = lo
        lo += 1
ans[0] = lo
print(ans)
