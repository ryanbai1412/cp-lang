// Source: Codeforces 2178A (Yes or Yes), https://codeforces.com/contest/2178/problem/A
// Problem: a string s of Y and N, 2 <= |s| <= 100. Repeatedly replace two adjacent characters by
//   their OR, never merging two Ys. Print YES if s can shrink to one character. Multitest.
// Solution: a Y can only merge with an N and stays a Y, so the number of Ys never changes. One
//   character left means at most one Y, and with at most one Y every merge is allowed. Count
//   the Ys, O(|s|).

@!multitest
let s: Str = read()
print(if s.count('Y') <= 1: "YES" else: "NO")
