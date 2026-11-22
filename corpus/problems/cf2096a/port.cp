// Codeforces 2096 A — Wonderful Sticks
//
// Build a permutation `ans[0..n]` of `1..=n` where `s[i] == '>'` means
// `ans[i + 1]` is larger than every earlier entry `ans[0..=i]` and `'<'` means
// smaller. Filling from the right (`i = n - 2` down to `0`) with unused range
// `[lo..=hi]`: `'>'` takes `hi` (strictly greater than everything left in
// `[lo..=hi - 1]`) and `'<'` takes `lo` (strictly smaller than `[lo + 1..=hi]`).
// After `n - 1` placements, `lo == hi` is the sole remaining value for `ans[0]`.

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
