// Source: CCC 2003 S4 (Substrings), https://dmoj.ca/problem/ccc03s4
// Problem: for each of N strings of 1 to 5000 alphanumeric characters, print the number of its
//   distinct substrings, the empty one included.
// Solution: each distinct non-empty substring is a prefix of some suffix; in suffix-array order
//   suffix i adds its length minus its LCP with the previous suffix, so the count is
//   n(n+1)/2 - sum(lcp) (`SuffixArray.distinct()`). Add 1 for the empty string. O(n) per string.

@!multitest
let s: Str = read()
print(1 + SuffixArray.new(s).distinct())
