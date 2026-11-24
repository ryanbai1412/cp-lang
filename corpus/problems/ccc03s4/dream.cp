// CCC 2003 S4 Substringy: count distinct substrings, the empty one included.
// Each suffix contributes its length minus its longest common prefix with the previous suffix.
@!multitest
let s: Str = read()
let sa = SuffixArray(s)
print(1 + sa.sa.sum(of: i => s.len - i) - sa.lcp.sum())
