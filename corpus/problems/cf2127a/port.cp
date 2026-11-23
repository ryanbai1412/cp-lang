// Source: Codeforces 2127A (Mix Mex Max), https://codeforces.com/contest/2127/problem/A
// Problem: n <= 100 values in 0..=100, -1 marking a blank (multitest). Can the blanks be filled
//   with non-negative integers so every window of 3 has mex = max - min? Print YES or NO.
// Solution: a window without 0 has mex 0, so it is constant; one with 0 has min 0 and needs
//   mex = max, impossible since max is in it and mex is not. So the array is one positive
//   constant: YES iff the fixed entries are all equal and positive. O(n).

@!multitest
let n: 3..=100 = read()
let a: [0..n]?(0..=100) = read(none: -1)
let fixed = [x for x in a if let x = x]
let ok = fixed.is_empty or (fixed[0] > 0 and fixed.all(x => x == fixed[0]))
print(yes(ok))
