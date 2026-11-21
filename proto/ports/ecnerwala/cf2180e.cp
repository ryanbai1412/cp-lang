// Codeforces 2180 E: count valid XOR values x > 0 mapping [l, r] onto itself;
// shifts below min(ctz(l), ctz(r + 1)) always work, and flipping the top split
// bit doubles the count iff l and r + 1 are symmetric around that boundary.
@!multitest
let (l, r0): (Int, Int) = read()
let r = r0 + 1
let half = 1 << (bit_width(l ^ r) - 1)
let sym = (l & (half - 1)) + (r & (half - 1)) == half
print(((1 + Int(sym)) << min(ctz(l), ctz(r))) - 1)
