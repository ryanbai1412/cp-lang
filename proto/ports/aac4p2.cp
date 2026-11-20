// cp: 0.1 71e98d9+dirty
// An Animal Contest 4 P2: for each query x, the first position i such that x
// is not a multiple of a[1..i] all at once (of their lcm), or -1.
const CAP = 2000000000                  // above every query: nothing divides past it
let n: 1..=1e6 = read()
let q: 1..=1e6 = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
let queries: [q]1..=1e9 = read()

var pre: [I.bounds]1..=CAP = 1          // lcm of everything before a bound, capped
for i in I:
    pre[i.r] = min(CAP, lcm(pre[i.l], a[i]))

for x in queries:
    // multiples of a longer prefix are multiples of every shorter one
    if let i = I.first_where(i => x % pre[i.r] != 0):
        print(i)
    else:
        print(-1)
