// expect: ok, output "ok"
// input: "1 300\n"
// model: ntt
// model: convolve
// model: convolve_all
// model: convolve_2d
// model: RelaxedConvolve
// model: SemiRelaxedConvolve
// std.ntt's products against the O(n m) double loop: ntt against the direct DFT sum, convolve
// on every index op and semiring (lengths past 60 so the NTT and three-prime paths run),
// convolve_all and convolve_2d against folded loops, and the online convolutions against the
// prefix sum after every push. Reads `seed n`; prints the first op that differs.
use std.ntt.{ntt, convolve, convolve_all, convolve_2d, RelaxedConvolve, SemiRelaxedConvolve, subset, Curved, IndexOp}
use std.math.{gcd, lcm}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>
type W = Mod<1_000_000_007>

// A user index op: (3 i + j) % 5, dropping pairs with i + j divisible by 3.
struct Skew

impl IndexOp for Skew:
    fn len(self, na: Int, nb: Int) -> Int = 5
    fn at(self, i: Int, j: Int) -> ?Int = if (i + j) % 3 == 0: None else: Some((i * 3 + j) % 5)

fn naive(a: List<Int>, b: List<Int>, at: (Int, Int) -> ?Int, len: Int) -> List<Int>:
    var c = List.filled(len, 0)
    for i in 0..a.len:
        for j in 0..b.len:
            if let t = at(i, j): c[t] += a[i] * b[j]
    c

fn naive_z(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty or b.is_empty: return []
    var c = List.filled(a.len + b.len - 1, Z(0))
    for i in 0..a.len:
        for j in 0..b.len: c[i + j] += a[i] * b[j]
    c

fn naive_semi(a: List<Int>, b: List<Int>, low: Bool) -> List<Int>:
    var c: List<Int> = []
    for k in 0..a.len + b.len - 1:
        var best: ?Int = None
        for i in 0..a.len:
            let j = k - i
            if 0 <= j < b.len:
                let v = a[i] + b[j]
                if best == None or (low and v < best!) or (not low and v > best!): best = Some(v)
        c.push(best!)
    c

// A convex (low) or concave sequence of length len.
fn curved(mut rng: Rng, len: Int, low: Bool) -> List<Int>:
    var ds = [rng.int(-20, 20) for _ in 0..len].sorted()
    if not low: ds = ds.reversed()
    var out = [rng.int(-20, 20)]
    for i in 1..len: out.push(out.last! + ds[i])
    out

fn dft(a: List<Z>, inverse: Bool) -> List<Z>:
    let n = a.len
    var w = Z(3) ** ((998_244_353 - 1) / n)
    if inverse: w = w.inv()
    var out: List<Z> = []
    for k in 0..n:
        var s = Z(0)
        for j in 0..n: s += a[j] * w ** (j * k % n)
        out.push(if inverse: s / Z(n) else: s)
    out

fn rev_bits(i: Int, n: Int) -> Int:
    var (r, x, m) = (0, i, 1)
    while m < n:
        r = r * 2 + x % 2
        x /= 2
        m *= 2
    r

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 9)
    let big = rng.int(0, 5) == 0
    let na = if big: rng.int(55, 90) else: rng.int(0, 8)
    let nb = if big: rng.int(55, 90) else: rng.int(0, 8)
    let a = [rng.int(-1000, 1000) for _ in 0..na]
    let b = [rng.int(-1000, 1000) for _ in 0..nb]
    if op == 0:
        let len = 1 << rng.int(0, 5)
        let x = [Z(rng.int(0, 998_244_352)) for _ in 0..len]
        var y = x.clone()
        ntt(mut y)
        var z = x.clone()
        ntt(mut z, bit_reversed: true)
        var back = z.clone()
        ntt(mut back, inverse: true, bit_reversed: true)
        var inv = x.clone()
        ntt(mut inv, inverse: true)
        if y != dft(x, false): bad = "op {k}: ntt({x})"
        elif any(z[i] != y[rev_bits(i, len)] for i in 0..len): bad = "op {k}: ntt({x}, bit_reversed: true)"
        elif back != x or inv != dft(x, true): bad = "op {k}: ntt({x}, inverse: true)"
    elif op == 1:
        let want = naive(a, b, (i, j) => Some(i + j), if na == 0 or nb == 0: 0 else: na + nb - 1)
        let za = [Z(v) for v in a]
        let zb = [Z(v) for v in b]
        let wa = [W(v) for v in a]
        let wb = [W(v) for v in b]
        if convolve(a, b) != want: bad = "op {k}: convolve({a}, {b})"
        elif convolve(za, zb) != [Z(v) for v in want]: bad = "op {k}: convolve over Z({a}, {b})"
        elif convolve(wa, wb) != [W(v) for v in want]: bad = "op {k}: convolve over W({a}, {b})"
        elif na == nb and na > 0:
            let cyc = naive(a, b, (i, j) => Some((i + j) % na), na)
            if convolve(a, b, cyclic: true) != cyc or convolve(za, zb, cyclic: true) != [Z(v) for v in cyc]: bad = "op {k}: convolve({a}, {b}, cyclic: true)"
    elif op == 2:
        let len = 1 << rng.int(0, 5)
        let x = [rng.int(-50, 50) for _ in 0..len]
        let y = [rng.int(-50, 50) for _ in 0..len]
        if convolve(x, y, on: ^) != naive(x, y, (i, j) => Some(i ^ j), len): bad = "op {k}: convolve({x}, {y}, on: ^)"
        elif convolve(x, y, on: |) != naive(x, y, (i, j) => Some(i | j), len): bad = "op {k}: convolve({x}, {y}, on: |)"
        elif convolve(x, y, on: &) != naive(x, y, (i, j) => Some(i & j), len): bad = "op {k}: convolve({x}, {y}, on: &)"
        elif convolve(x, y, on: subset) != naive(x, y, (i, j) => if i & j == 0: Some(i | j) else: None, len): bad = "op {k}: convolve({x}, {y}, on: subset)"
    elif op == 3:
        if na > 0 and nb > 0:
            let len = max(na, nb)
            let g = naive(a, b, (i, j) => if i >= 1 and j >= 1: Some(gcd(i, j)) else: None, len)
            let l = naive(a, b, (i, j) => if i >= 1 and j >= 1 and lcm(i, j) < len: Some(lcm(i, j)) else: None, len)
            if convolve(a, b, on: gcd) != g: bad = "op {k}: convolve({a}, {b}, on: gcd)"
            elif convolve(a, b, on: lcm) != l: bad = "op {k}: convolve({a}, {b}, on: lcm)"
            elif convolve(a, b, on: Skew()) != naive(a, b, (i, j) => if (i + j) % 3 == 0: None else: Some((i * 3 + j) % 5), 5): bad = "op {k}: convolve({a}, {b}, on: Skew())"
    elif op == 4:
        let low = rng.int(0, 1) == 0
        let (x, y) = (curved(mut rng, max(na, 1), low), curved(mut rng, max(nb, 1), low))
        let want = naive_semi(x, y, low)
        let wild = [rng.int(-50, 50) for _ in 0..max(na, 1)]
        if low:
            if convolve(x, y, over: Semiring(min, +), shape: Curved.Both) != want: bad = "op {k}: min-plus Both({x}, {y})"
            elif convolve(wild, y, over: Semiring(min, +), shape: Curved.Right) != naive_semi(wild, y, true): bad = "op {k}: min-plus Right({wild}, {y})"
            elif convolve(wild, y, over: Semiring(min, +), shape: Curved.Neither) != naive_semi(wild, y, true): bad = "op {k}: min-plus Neither({wild}, {y})"
        else:
            if convolve(x, y, over: Semiring(max, +), shape: Curved.Both) != want: bad = "op {k}: max-plus Both({x}, {y})"
            elif convolve(wild, y, over: Semiring(max, +), shape: Curved.Right) != naive_semi(wild, y, false): bad = "op {k}: max-plus Right({wild}, {y})"
    elif op == 5:
        let cnt = rng.int(0, 6)
        let ps = [[Z(rng.int(0, 9)) for _ in 0..rng.int(if rng.int(0, 9) == 0: 0 else: 1, 5)] for _ in 0..cnt]
        var want = [Z(1)]
        for p in ps: want = naive_z(want, p)
        if convolve_all(ps) != want: bad = "op {k}: convolve_all({ps})"
    elif op == 6:
        let ra = rng.int(0, 4)
        let rb = rng.int(0, 4)
        let x = [[Z(rng.int(0, 9)) for _ in 0..rng.int(0, 4)] for _ in 0..ra]
        let y = [[Z(rng.int(0, 9)) for _ in 0..rng.int(0, 4)] for _ in 0..rb]
        let wa = max(r.len for r in x) ?? 0
        let wb = max(r.len for r in y) ?? 0
        var want: List<List<Z>> = []
        if wa > 0 and wb > 0:
            want = [List.filled(wa + wb - 1, Z(0)) for _ in 0..ra + rb - 1]
            for i in 0..ra:
                for c in 0..x[i].len:
                    for j in 0..rb:
                        for d in 0..y[j].len: want[i + j][c + d] += x[i][c] * y[j][d]
        if convolve_2d(x, y) != want: bad = "op {k}: convolve_2d({x}, {y})"
    elif op == 7:
        var rc = RelaxedConvolve<998_244_353>()
        var (xs, ys) = (List<Z>(), List<Z>())
        for t in 0..rng.int(1, 80):
            // feed back the last output, the online-DP shape
            let xa = if t > 0 and rng.int(0, 1) == 0: Z(rng.int(0, 99)) + Z(t) else: Z(rng.int(0, 998_244_352))
            xs.push(xa)
            ys.push(Z(rng.int(0, 998_244_352)))
            let got = rc.push(xa, ys.last!)
            var want = Z(0)
            for i in 0..=t: want += xs[i] * ys[t - i]
            if bad == "" and got != want: bad = "op {k}: RelaxedConvolve push {t}"
    else:
        let fixed = [Z(rng.int(0, 998_244_352)) for _ in 0..rng.int(0, 40)]
        var sc = SemiRelaxedConvolve<998_244_353>(fixed)
        var xs = List<Z>()
        for t in 0..rng.int(1, 80):
            let xa = Z(rng.int(0, 998_244_352))
            xs.push(xa)
            let got = sc.push(xa)
            var want = Z(0)
            for i in 0..=t:
                if t - i < fixed.len: want += xs[i] * fixed[t - i]
            if bad == "" and got != want: bad = "op {k}: SemiRelaxedConvolve({fixed.len}) push {t}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
