# CCC 2003 S4 Substringy: count distinct substrings, the empty one included.
# Each suffix contributes its length minus its longest common prefix with the previous suffix.
for s in read [t] Str:
    let sa = SuffixArray(s)
    print(1 + sa.order.sum(of: i => s.len - Int(i)) - sa.lcp.sum())
