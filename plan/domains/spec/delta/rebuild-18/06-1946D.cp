// CF1946D Birthday Gift. The OR of the pieces is at most x iff it is a submask
// of x or of some x with one set bit cleared and every lower bit set; for a
// mask, the most pieces is the number of prefix XORs that are submasks of it.
@!multitest
let n: 1..=1e5 = read()
let x: 0..=1073741823 = read()
let a = read<[n]0..=1073741823>()
let pre = a.scan(^)
let pieces = (m: Int) =>
    if (pre[n] | m) != m: -1
    else: (1..=n).count(k => (pre[k] | m) == m)
var best = pieces(x)
for b in 0..30:
    if ((x >> b) & 1) == 1:
        best max= pieces(((x >> (b + 1)) << (b + 1)) | ((1 << b) - 1))
print(best)
