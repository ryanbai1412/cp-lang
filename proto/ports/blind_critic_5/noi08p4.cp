// Runnable CP Lang v2.2 port of noi08p4 (NOI '08 P4 - Transmitting)
// Original C++: corpus/submissions/5144322.cpp
// Runs directly under the Rust `cpl run` and `cpl build` (c521b2c).
// Workarounds vs blind first draft (noi08p4_draft.cp):
// - Rust c521b2c uses `F64` instead of locked v2.2 `F64`, and `print(ans, digits: 2)` instead of `"{ans:.2f}"`
// - No `@memo` on nested closures or `for u in V if cond:` -> explicit memo table & `if`
// - Uses real v2.2 `swap(mut a, mut b)` and `List<F64>` value semantics

struct KS:
    msz: Int
    v: List<F64>

    fn make(msz: Int) -> KS:
        KS(msz: msz, v: [0.0])

    fn update(mut self, arr: List<F64>):
        let n = self.v.len
        let m = arr.len
        let lim = min(self.msz, n + m)
        var nv: List<F64> = []
        for _ in 0..lim:
            nv.push(0.0)
        for i in 0..n:
            for j in 0..m:
                if i + j < self.msz:
                    nv[i + j] max= self.v[i] + arr[j]
        for i in 0..(lim - 1):
            nv[i + 1] = max(nv[i], nv[i + 1])
        self.v = nv

fn shiftr(arr: List<F64>, len: Int) -> List<F64>:
    var out: List<F64> = []
    for _ in 0..len:
        out.push(0.0)
    for x in arr:
        out.push(x)
    out

fn vor(a_in: List<F64>, b_in: List<F64>) -> List<F64>:
    var a = a_in
    var b = b_in
    if a.len < b.len:
        swap(mut a, mut b)
    for i in 0..b.len:
        a[i] max= b[i]
    a

let n: Int = read()
let m: Int = read()
let kv: F64 = read()

var par: List<Int> = []
for _ in 0..n:
    let p: Int = read()
    par.push(p - 1)

var arr: List<F64> = []
for _ in 0..n:
    let a: F64 = read()
    arr.push(a)

var cyc: List<Int> = []
var cur_u = 0
while true:
    cyc.push(cur_u)
    cur_u = par[cur_u]
    if cur_u == 0:
        break

var kpow: List<F64> = [1.0]
for i in 0..(n + 4):
    kpow.push(kpow[i] * kv)

var graph: List<List<Int>> = []
var memo_done: List<Bool> = []
var memo_val: List<List<F64>> = []
var cmul = 1.0
let stride = n + 2

fn dp(u: Int, cdist: Int) -> List<F64>:
    let slot = u * stride + cdist
    if memo_done[slot]:
        return memo_val[slot]
    var ks = KS.make(m + 1)
    ks.update([arr[u] * kpow[cdist] * cmul])
    for v in graph[u]:
        let a = dp(v, cdist + 1)
        let b = shiftr(dp(v, 1), 1)
        ks.update(vor(a, b))
    memo_done[slot] = true
    memo_val[slot] = ks.v
    ks.v

var ans = 0.0
for clen in 2..=cyc.len:
    var ccyc: List<Int> = []
    var icyc: List<Bool> = []
    for _ in 0..n:
        icyc.push(false)
    for i in 0..clen:
        ccyc.push(cyc[i])
        icyc[cyc[i]] = true

    graph.clear()
    for _ in 0..n:
        var empty: List<Int> = []
        graph.push(empty)
    for i in 0..n:
        if not icyc[i]:
            graph[par[i]].push(i)

    memo_done.clear()
    memo_val.clear()
    for _ in 0..(n * stride):
        memo_done.push(false)
        var empty: List<F64> = []
        memo_val.push(empty)

    // Reverse ccyc[1..] in place
    var lo = 1
    var hi = clen - 1
    while lo < hi:
        ccyc.swap(lo, hi)
        lo += 1
        hi -= 1

    cmul = 1.0 / (1.0 - kpow[clen])
    let budget = m + Int(clen == cyc.len)
    if budget > 0:
        var cans = KS.make(budget)
        for i in 0..clen:
            cans.update(dp(ccyc[i], i))
        if cans.v.len > 0:
            ans max= cans.v[cans.v.len - 1]

print(ans, digits: 2)
