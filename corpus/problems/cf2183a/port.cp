// Codeforces 2183 A — Binary Array Game
//
// A move replaces a subarray of length >= 2 with `1 - min(subarray)`; Alice
// wins iff the final single element is `0`.
// - If `a[0] == 1` or `a[n - 1] == 1`: Alice collapses the complementary `n - 1`
//   elements (or the whole array if all `1`s), reaching either `[0]` or `[1, 1]`
//   where Bob's only move leaves `[0]`, so Alice wins.
// - If `a[0] == 0` and `a[n - 1] == 0`: collapsing the whole array yields `[1]`
//   (Bob wins), and any proper subarray leaves a `0` at an end so Bob collapses
//   the whole array to `[1]` on the next turn.

@!multitest
let n: 3..=100 = read()
let a: [0..n]0..=1 = read()
print(if a.first! == 0 and a.last! == 0: "Bob" else: "Alice")
