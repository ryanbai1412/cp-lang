// Source: Codeforces 2152A (Increase or Smash), https://codeforces.com/contest/2152/problem/A
// limits: 1000 ms, 1024 MB
// Problem: start from n <= 100 zeros. Increase adds one x > 0 to every element; Smash sets any
//   chosen elements to 0. Print the fewest operations that reach the target a (1 <= a[i] <= 100).
//   Multitest.
// Solution: an element ends at the sum of the Increases after its last Smash, so k distinct
//   values need k Increases, with a Smash between each two (adjacent Increases merge): 2k - 1,
//   reached by smashing the elements that end lower before each later Increase. k is
//   `Set(a).len`, O(n).
// Bounds: n, a: 1..=100

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=100 = read()
print(2 * Set(a).len - 1)
