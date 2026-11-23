// CF 1858D Trees and Segments
// ones[j, c]: longest run of 1s inside the prefix before cut c using <= j
// flips (two pointers: at most j zeros in the window); back[j, c] the same
// for the suffix after c (hand-rolled: maximal over Window does not
// compile, see F2). Every zero block l..r of cost c leaves k - c
// flips for a 1-run on either side.
@!multitest
let (n, k): (1..=3000, 0..=n) = read()
let s = read<Str>()
domain P = 0..n
domain J = 0..=k
domain Len = 0..=n
let zero: [P]Int = [P] i => if s[Int(i)] == '0': 1 else: 0
var pre: [J, P.cuts]Int = 0
var suf: [J, P.cuts]Int = 0
for j in J:
    var z = 0
    var lo = 0
    for i in P:
        z += zero[i]
        while z > Int(j):
            z -= zero[P(lo)]
            lo += 1
        pre[j, i.r] = max(pre[j, i.l], Int(i) + 1 - lo)
    z = 0
    var hi = n - 1
    for i in P.rev:
        z += zero[i]
        while z > Int(j):
            z -= zero[P(hi)]
            hi -= 1
        suf[j, i.l] = max(suf[j, i.r], hi - Int(i) + 1)
var best: [Len]?Int = None
best[Len(0)] = pre[J(k), P.cuts.last!]
for i in P:
    var cost = 0
    for e in P if e >= i:
        cost += 1 - zero[e]
        if cost > k: break
        let rem = J(k - cost)
        best[Len((i..=e).len)] max= max(pre[rem, i.l], suf[rem, e.r])
var out = List<Int>()
for a in 1..=n:
    var v: ?Int = None
    for l in Len:
        if let b = best[l]: v max= a * Int(l) + b
    out.push(v!)
print(out)
