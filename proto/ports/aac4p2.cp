// An Animal Contest 4 P2: for each query x, the first position i such that x
// is not a multiple of a[1..i] all at once (of their lcm), or -1.

const CAP = 2e9  // above every query: nothing divides past it
let (n, q): (1..=1e6, 1..=1e6) = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()

var pre: [I]Int = 1  // lcm of a[..=i], capped
var all = 1
for i in I:
    all = min(CAP, lcm(all, a[i]))
    pre[i] = all

for _ in 0..q:
    let x: 1..=1e9 = read()
    // multiples of a longer prefix are multiples of every shorter one
    if let i = bsearch_first(I.first.., i => x % pre[i] != 0): print(i) else: print(-1)
