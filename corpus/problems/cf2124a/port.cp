// Source: Codeforces 2124A (Deranged Deletions), https://codeforces.com/contest/2124/problem/A
// limits: 1000 ms, 250 MB
// Problem: an array is a derangement if it differs from its sorted copy at every position.
//   Given n <= 100 values 1 <= a_i <= n, delete some elements so a non-empty derangement
//   remains: print NO, or YES, its length and its elements. Multitest.
// Solution: if a is non-decreasing every subsequence is sorted, so NO. Otherwise some adjacent
//   pair has x > y, and [x, y] differs from its sorted [y, x] at both positions: print it. O(n).

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=100 = read()
for (x, y) in a.adjacent if x > y:
    print("YES")
    print(2)
    print(x, y)
    return
print("NO")
