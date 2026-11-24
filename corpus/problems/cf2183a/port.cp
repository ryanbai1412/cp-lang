// Source: Codeforces 2183A (Binary Array Game), https://codeforces.com/contest/2183/problem/A
// limits: 1000 ms, 256 MB
// Problem: a 0/1 array of n <= 100 (multitest). Alice and Bob alternate, Alice first, replacing
//   a subarray of length >= 2 by 1 - its min, until one element is left; Alice wins if it is 0.
//   Print the winner.
// Solution: if an end is 1, Alice collapses an all-1 array to [0], else the other n - 1
//   elements (which hold a 0) to 1, leaving [1, 1], which Bob must turn into [0]. If both ends
//   are 0, collapsing everything gives 1 and any shorter move keeps a 0 at an end, so Bob then
//   collapses all to 1. Alice iff a[0] or a[n - 1] is 1: O(1).

@!multitest
let n: 3..=100 = read()
let a: [0..n]0..=1 = read()
print(if a.first! == 0 and a.last! == 0: "Bob" else: "Alice")
