// UCup 4 Stage 25 (Zagreb) H — Median Single-Character Deletion
//
// Given `s` (`1 <= |s| <= 500_000`), compare deleting position `i` vs any
// later position `j > i`: both strings agree before `i`, then compare `s[i + 1]`
// against `s[i]` (skipping equal runs to the first `s[k] != s[i]`).
// Scanning `i` right-to-left, `go_right` records whether the next distinct
// character after `i` is strictly larger (`s[k] > s[i]`):
// - If `go_right` is true (deleting `i` raises the first differing char), `i`
//   ranks after all later deletions (`right.push(i)`).
// - If `go_right` is false (deleting `i` lowers the first differing char), `i`
//   ranks before all later deletions (`left.push(i)`).
// Thus `left.rev + right` is the exact lexicographical order of all `n`
// deletions, and index `n / 2` is the median.

@!multitest
let s: Str = read()
let n = s.len
var left: List<Int> = []
var right: List<Int> = []
var go_right = false
for i in (0..n).rev:
    if go_right: right.push(i)
    else: left.push(i)
    if i > 0 and s[i] != s[i - 1]: go_right = s[i - 1] > s[i]
let del = (left.rev + right)[n / 2]
print(s[0..del] + s[del + 1..n])
