// Codeforces 2127 A — Mix Mex Max
//
// In the input, `-1` is a wildcard that can be replaced by any non-negative
// integer. Every triple `{x, y, z}` satisfies `MEX(x, y, z) == max(x, y, z) -
// min(x, y, z)` iff all three elements are equal and positive (`0` makes
// `MEX >= 1` while `max - min == 0` when equal, and distinct values cannot keep
// every adjacent triple valid across the array). Thus a valid completion exists
// iff all fixed (`!= none`) entries are positive and share one common value.

@!multitest
let n: 3..=100 = read()
let a: [0..n]?(0..=100) = read(none: -1)
let fixed = [x for x in a if let x = x]
let ok = fixed.is_empty or (fixed[0] > 0 and fixed.all(x => x == fixed[0]))
print(if ok: "YES" else: "NO")
