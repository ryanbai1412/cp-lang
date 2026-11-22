// Codeforces 2211 A — Antimedian Deletion
//
// Each operation deletes the median of a chosen length-3 subarray (never the
// minimum or maximum of the triple), reducing the array length by 1 until
// length 2. For `n == 1`, no operation applies so the answer is `[1]`. For
// `n >= 2`, any target element `p[i]` can survive down to length 2 (by always
// picking a triple containing `p[i]` where `p[i]` is an extremum, or deleting
// outside `p[i]`), while length 1 is unreachable, so every position answers `2`.

@!multitest
let n: 1..=100 = read()
let _: [0..n]1..=n = read()
print(List.filled(n, if n == 1: 1 else: 2))
