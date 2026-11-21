// Codeforces 2066 B: White Magic (dream). Zero-free sequences are magical;
// keeping only the leftmost zero is the one candidate for one more.
@!multitest
let n: 1..=2e5 = read()
let a: [n]0..=1e9 = read()
let zeros = a.count(where: x => x == 0)
guard let first = a.find(0) else: return print(n)
let b = a.positions.filter(i => a[i] != 0 or i == first).map(a)   // its own index space with cuts
let pre = b.prefix_min()                                           // [b.Pos]Int
let mex = b.suffix_mex(at: .cuts)                                  // [b.Cut]Int, empty suffix last
print(n - zeros + Int(all(pre[c.before] >= mex[c] for c in b.cuts.inner)))
