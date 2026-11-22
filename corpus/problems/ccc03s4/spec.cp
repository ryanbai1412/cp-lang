// ccc03s4 Substrings: the number of distinct substrings, counting the empty one. A suffix
// array counts the non-empty ones as n(n+1)/2 minus the LCP sum.
@!multitest
let s = read<Str>()
print(SuffixArray(s).distinct() + 1)
