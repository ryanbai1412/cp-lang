// Codeforces 2135 A — Against the Difference
//
// A neat subsequence is a concatenation of blocks where each block consists of
// `x` equal copies of `x`. Let `dp[j]` be the maximum length of a neat
// subsequence inside the suffix `a[j..n]`.
//
// Pre-collect `inds[x]` as the ascending list of indices where `a[k] == x`.
// Sweeping `i` from `n - 1` down to `0` maintains the invariant that `inds[x]`
// holds all occurrences of `x` in `0..=i`, with `i` at the back (`inds[x].last`).
// Thus if `inds[x].len >= x`, the `x`-th occurrence ending at `i` starts at
// `start = inds[x][inds[x].len - x]`, giving the block `start..=i` of length
// `x` followed by any neat subsequence in `a[i + 1..n]`:
// `dp[start] max= dp[i + 1] + x`, before popping `i` from `inds[x]`.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=n = read()
var inds: [1..=n]List<Int> = []
for (i, x) in a.items: inds[x].push(i)

var dp: [0..=n]Int = 0
for i in (0..n).rev:
    dp[i] max= dp[i + 1]
    let x = a[i]
    if inds[x].len >= x:
        let start = inds[x][inds[x].len - x]
        dp[start] max= dp[i + 1] + x
    inds[x].pop()
print(dp[0])
