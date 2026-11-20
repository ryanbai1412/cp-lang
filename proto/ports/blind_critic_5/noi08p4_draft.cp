// Problem: noi08p4 (NOI 2008 P4 - Olympic Logistics)
// Original: corpus/submissions/5144322.cpp
// Blind Critic #5 — First Draft (strictly from docs/one-page-cheat-sheet.md v2.2)

struct KS:
    msz: Int
    v: List<F64>

    fn new(msz: Int) -> KS:
        return KS(msz: msz, v: [0.0])

    fn update(mut self, arr: List<F64>):
        let n = self.v.len
        let m = arr.len
        var nv = List.filled(min(self.msz, n + m), 0.0)
        for i in 0..n:
            for j in 0..m if i + j < self.msz:
                nv[i + j] max= self.v[i] + arr[j]
        for i in 0..nv.len - 1:
            nv[i + 1] max= nv[i]
        self.v = nv

fn shiftr(arr: List<F64>, len: Int) -> List<F64>:
    return List.filled(len, 0.0) + arr

fn vor(a: List<F64>, b: List<F64>) -> List<F64>:
    var (big, small) = if a.len >= b.len: (a, b) else: (b, a)
    for i in 0..small.len:
        big[i] max= small[i]
    return big

let (n, m, kv): (1..=60, 0..=60, F64) = read()
domain V = 1..=n
let par: [V]V = read()
let arr: [V]F64 = read()

var cyc: List<V> = []
var cur = V.first
while true:
    cyc.push(cur)
    cur = par[cur]
    if cur == V.first:
        break

var kpow: [0..=n]F64 = 1.0
for i in 0..n:
    kpow[i + 1] = kpow[i] * kv

var ans = 0.0
for clen in 2..=cyc.len:
    var ccyc = List(cyc[0..clen])
    var icyc: [V]Bool = false
    for u in ccyc:
        icyc[u] = true

    var g: [V]List<V> = []
    for u in V if not icyc[u]:
        g[par[u]].push(u)

    reverse(mut ccyc[1..])
    let cmul = 1.0 / (1.0 - kpow[clen])

    @memo
    fn dp(u: V, cdist: 0..=n) -> List<F64>:
        var ks = KS.new(m + 1)
        ks.update([arr[u] * kpow[cdist] * cmul])
        for v in g[u]:
            ks.update(vor(dp(v, cdist + 1), shiftr(dp(v, 1), 1)))
        return ks.v

    var cans = KS.new(m + Int(clen == cyc.len))
    for (i, u) in ccyc.items:
        cans.update(dp(u, i))
    ans max= cans.v.last!

print("{ans:.2f}")
