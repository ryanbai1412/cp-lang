// expect: ok, output "ok"
// input: "1 300\n"
// model: Mod
// model: Comb
// model: Barrett
// model: PrimePowerComb
// model: ModComb
// Mod<P> against Int arithmetic and loops over 0..P, Comb against Pascal's triangle mod a prime,
// Barrett against `%`, and PrimePowerComb and ModComb against Pascal's triangle mod a composite:
// seeded random values and queries. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.mod.{Barrett, Comb, ModComb, PrimePowerComb}
type Z = Mod<13>
type W = Mod<12>
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var pas = [[0 for _ in 0..41] for _ in 0..41]
for i in 0..41:
    pas[i][0] = 1
    for j in 1..=i: pas[i][j] = pas[i - 1][j - 1] + (if j < i: pas[i - 1][j] else: 0)
let c13 = Comb<Z>()
let cbig = Comb<Mod<1_000_000_007>>(5)
let mcs = [(m, ModComb(m)) for m in [1, 12, 36, 98, 1000]]
let ppc = [((p, e), PrimePowerComb(p, e)) for (p, e) in [(2, 3), (3, 2), (5, 1), (2, 5)]]
var bad = ""
for k in 0..n:
    let a = rng.int(-100, 100)
    let b = rng.int(-100, 100)
    let (x, y) = (Z(a), Z(b))
    let r = (a % 13 + 13) % 13
    let s = (b % 13 + 13) % 13
    if x.v != r or Z.from_int(a) != x or Z.new(a) != x: bad = "op {k}: Z({a})"
    elif (x + y).v != (r + s) % 13 or (x - y).v != (r - s + 13) % 13 or (x * y).v != r * s % 13 or (-x).v != (13 - r) % 13:
        bad = "op {k}: arithmetic on {a}, {b}"
    if bad != "": break
    let e = rng.int(0, 30)
    var p = 1
    for _ in 0..e: p = p * r % 13
    if (x ** e).v != p: bad = "op {k}: {a} ** {e}"
    // Inverses, square roots and logs against a scan of every value.
    var inv13: ?Z = None
    for t in 0..13:
        if r * t % 13 == 1: inv13 = Some(Z(t))
    if x.try_inv() != inv13: bad = "op {k}: try_inv({a})"
    elif s != 0 and x / y != x * Z(b).inv(): bad = "op {k}: {a} / {b}"
    var inv12: ?W = None
    for t in 0..12:
        if (a % 12 + 12) % 12 * t % 12 == 1 % 12: inv12 = Some(W(t))
    if W(a).try_inv() != inv12: bad = "op {k}: Mod<12>.try_inv({a})"
    elif inv12 != None and W(a).inv() != inv12!: bad = "op {k}: Mod<12>.inv({a})"
    let root = x.sqrt()
    let has_root = any(t * t % 13 == r for t in 0..13)
    if (root != None) != has_root or (root != None and root! * root! != x): bad = "op {k}: sqrt({a})"
    var lg: ?Int = None
    var acc = 1 % 13
    for t in 0..13:
        if lg == None and acc == s: lg = Some(t)
        acc = acc * r % 13
    let got = x.log(y)
    if (got == None) != (lg == None) or (got != None and x ** got! != y): bad = "op {k}: log({a}, {b}) = {got ?? -1} want {lg ?? -1}"
    let xs = [Z(rng.int(1, 12)) for _ in 0..rng.int(0, 5)]
    if Z.inv_all(xs) != [q.inv() for q in xs]: bad = "op {k}: inv_all"
    if bad != "": break
    // Comb mod 13 and mod 1e9+7 against Pascal's triangle.
    let nn = rng.int(0, 40)
    let kk = rng.int(-2, 42)
    let want = if 0 <= kk <= nn: pas[nn][kk] else: 0
    var f = 1
    for t in 1..=nn: f = f * t % 13
    var arr = 1
    for t in 0..kk: arr = arr * (nn - t) % 13
    if c13.choose(nn, kk).v != want % 13 or cbig.choose(nn, kk).v != want % 1_000_000_007:
        bad = "op {k}: choose({nn}, {kk})"
    elif c13.fact(nn).v != f or (nn < 13 and c13.inv_fact(nn) * c13.fact(nn) != Z(1)):
        bad = "op {k}: fact({nn})"
    elif 1 <= nn < 13 and c13.inv(nn) * Z(nn) != Z(1):
        bad = "op {k}: inv({nn})"
    elif kk >= 0 and c13.arrange(nn, kk).v != (if kk <= nn: arr else: 0):
        bad = "op {k}: arrange({nn}, {kk})"
    elif nn + kk - 1 <= 40 and cbig.multichoose(nn, kk).v != (if nn == 0: 1 elif kk <= 0: 0 else: pas[nn + kk - 1][nn] % 1_000_000_007):
        bad = "op {k}: multichoose({nn}, {kk})"
    elif nn <= 20 and cbig.catalan(nn).v != pas[2 * nn][nn] / (nn + 1) % 1_000_000_007:
        bad = "op {k}: catalan({nn})"
    if bad != "": break
    var der = [1, 0]
    for t in 2..=nn: der.push((t - 1) * (der[t - 1] + der[t - 2]) % 1_000_000_007)
    if cbig.derange(nn).v != der[nn]: bad = "op {k}: derange({nn})"
    let parts = [rng.int(0, 4) for _ in 0..rng.int(0, 4)]
    var tot = 0
    var multi = 1
    for q in parts:
        tot += q
        multi *= pas[tot][q]
    if cbig.multinomial(parts).v != multi: bad = "op {k}: multinomial({parts})"
    // Barrett against `%`.
    let m = rng.int(1, 2_000_000_000)
    let br = Barrett(m)
    let (u, w) = (rng.int(0, m - 1), rng.int(0, m - 1))
    if br.mul(u, w) != Int(I128(u) * I128(w) % I128(m)): bad = "op {k}: Barrett({m}).mul({u}, {w})"
    elif br.reduce(U64(u)) != u: bad = "op {k}: Barrett({m}).reduce({u})"
    if bad != "": break
    // PrimePowerComb and ModComb against Pascal's triangle mod a composite.
    for ((pp, ee), q) in ppc:
        var pe = 1
        for _ in 0..ee: pe *= pp
        if bad == "" and q.choose(nn, kk) != want % pe: bad = "op {k}: PrimePowerComb({pp}, {ee}).choose({nn}, {kk})"
    for (mm, q) in mcs:
        let wm = if mm == 1: 0 else: want % mm
        if bad == "" and q.choose(nn, kk) != wm: bad = "op {k}: ModComb({mm}).choose({nn}, {kk})"
    if bad != "": break
print(if bad == "": "ok" else: bad)
