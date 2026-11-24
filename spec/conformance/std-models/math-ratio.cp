// expect: ok, output "ok"
// input: "1 300\n"
// model: Ratio
// model: best_rational
// model: continued_fraction
// model: BigInt
// std.math's exact numbers against plain Int arithmetic: Ratio against a reduced (p, q) pair,
// best_rational against a scan of every denominator, continued_fraction by folding it back,
// and BigInt against Int where the values fit, plus divmod and round-trip identities on long
// decimal strings. Reads `seed n`; prints the first op that differs.
use std.math.{Ratio, best_rational, continued_fraction, BigInt}
use std.rand.{Rng}

fn g(a: Int, b: Int) -> Int:
    var (x, y) = (abs(a), abs(b))
    while y != 0: (x, y) = (y, x % y)
    x

fn red(p: Int, q: Int) -> Str:
    let d = g(p, q)
    if q < 0: "{-p / d}/{-q / d}" else: "{p / d}/{q / d}"

// The closest p / q to num / den with 1 <= q <= lim: least distance, then denominator, then value.
fn best_brute(num: Int, den: Int, lim: Int) -> Str:
    let (x, y) = if den < 0: (-num, -den) else: (num, den)
    var (bp, bq) = (0, 0)
    for q in 1..=lim:
        let f = x * q / y
        for p in [f, f + 1]:
            if g(p, q) != 1: continue
            if bq == 0: (bp, bq) = (p, q)
            else:
                let dn = abs(x * q - p * y) * bq
                let db = abs(x * bq - bp * y) * q
                if dn < db or (dn == db and q == bq and p < bp): (bp, bq) = (p, q)
    "{bp}/{bq}"

fn fold_cf(cf: List<Int>) -> (Int, Int):
    var (p, q) = (cf.last!, 1)
    for i in (0..cf.len - 1).rev: (p, q) = (cf[i] * p + q, p)
    (p, q)

fn digits_str(mut rng: Rng, len: Int) -> Str = Str([Char(48 + rng.int(0, 9)) for _ in 0..len])

fn strip(s: Str) -> Str:
    let neg = s.len > 0 and s[0] == '-'
    var i = if neg: 1 else: 0
    while i < s.len - 1 and s[i] == '0': i += 1
    let body = Str([s[j] for j in i..s.len])
    if neg and body != "0": "-" + body else: body

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 7)
    let (a, b, c, d) = (rng.int(-30, 30), rng.int(-30, 30), rng.int(-30, 30), rng.int(-30, 30))
    let b1 = if b == 0: 1 else: b
    let d1 = if d == 0: 1 else: d
    if op == 0:
        let x = Ratio.new(a, b1)
        let y = Ratio.new(c, d1)
        let q = b1 * d1
        if "{x}" != red(a, b1) or "{Ratio.from_int(a)}" != red(a, 1) or Ratio.new(a) != Ratio.from_int(a): bad = "op {k}: Ratio.new({a}, {b1})"
        elif "{x + y}" != red(a * d1 + c * b1, q) or "{x - y}" != red(a * d1 - c * b1, q): bad = "op {k}: {x} +- {y}"
        elif "{x * y}" != red(a * c, q) or "{-x}" != red(-a, b1): bad = "op {k}: {x} * {y}"
        elif c != 0 and "{x / y}" != red(a * d1, b1 * c): bad = "op {k}: {x} / {y}"
    elif op == 1:
        let x = Ratio.new(a, b1)
        let y = Ratio.new(c, d1)
        let want = a * b1 * d1 * d1 < c * d1 * b1 * b1
        if (x < y) != want or (x == y) != (a * d1 == c * b1): bad = "op {k}: {x} < {y}"
        elif x.floor != (a * (if b1 < 0: -1 else: 1)) / abs(b1): bad = "op {k}: {x}.floor"
        elif x.ceil != -((-a * (if b1 < 0: -1 else: 1)) / abs(b1)): bad = "op {k}: {x}.ceil"
        elif "{x.abs}" != red(abs(a), abs(b1)): bad = "op {k}: {x}.abs"
        elif a != 0 and "{x.inv}" != red(b1, a): bad = "op {k}: {x}.inv"
        elif x.p != (if b1 < 0: -a else: a) / g(a, b1): bad = "op {k}: {x}.p"
    elif op == 2:
        let num = rng.int(-400, 400)
        let den = rng.int(-60, 60)
        let lim = rng.int(1, 25)
        if den != 0 and "{best_rational(num, den, lim)}" != best_brute(num, den, lim): bad = "op {k}: best_rational({num}, {den}, {lim})"
    elif op == 3:
        let p = rng.int(-5000, 5000)
        let q = rng.int(1, 3000)
        let cf = continued_fraction(p, q)
        let (fp, fq) = fold_cf(cf)
        let canon = all(cf[i] > 0 for i in 1..cf.len) and (cf.len == 1 or cf.last! >= 2)
        if not canon or "{fp}/{fq}" != red(p, q) or fq <= 0: bad = "op {k}: continued_fraction({p}, {q}) == {cf}"
    elif op == 4:
        let x = rng.int(-2_000_000_000_000_000_000, 2_000_000_000_000_000_000)
        let y = rng.int(-2_000_000_000_000_000_000, 2_000_000_000_000_000_000)
        let (bx, by) = (BigInt.from_int(x), BigInt.from_int(y))
        if bx.to_str() != "{x}" or "{bx}" != "{x}" or BigInt.from_str("{x}") != bx: bad = "op {k}: BigInt.from_int({x})"
        elif bx + by != BigInt.from_int(x + y) or bx - by != BigInt.from_int(x - y) or -bx != BigInt.from_int(-x): bad = "op {k}: BigInt {x} +- {y}"
        elif (bx < by) != (x < y) or (bx == by) != (x == y) or bx.abs() != BigInt.from_int(abs(x)): bad = "op {k}: BigInt {x} < {y}"
        elif y != 0 and (bx.divmod(by) != (BigInt.from_int(x / y), BigInt.from_int(x % y)) or bx / by != BigInt.from_int(x / y)): bad = "op {k}: BigInt {x} divmod {y}"
    elif op == 5:
        let x = rng.int(-3_000_000_000, 3_000_000_000)
        let y = rng.int(-3_000_000_000, 3_000_000_000)
        let (bx, by) = (BigInt.from_int(x), BigInt.from_int(y))
        if bx * by != BigInt.from_int(x * y): bad = "op {k}: BigInt {x} * {y}"
        elif BigInt.gcd(bx, by) != BigInt.from_int(g(x, y)): bad = "op {k}: BigInt.gcd({x}, {y})"
    elif op == 6:
        let len = rng.int(1, 40)
        let s = (if rng.int(0, 1) == 0: "-" else: "") + Str(['0' for _ in 0..rng.int(0, 3)]) + digits_str(mut rng, len)
        let x = BigInt.from_str(s)
        if x.to_str() != strip(s): bad = "op {k}: BigInt.from_str({s})"
    else:
        let (lx, ly) = (rng.int(1, 40), rng.int(1, 25))
        let x = BigInt.from_str((if rng.int(0, 1) == 0: "-" else: "") + digits_str(mut rng, lx))
        var y = BigInt.from_str((if rng.int(0, 1) == 0: "-" else: "") + digits_str(mut rng, ly))
        if y == BigInt.from_int(0): y = BigInt.from_int(7)
        let (q, r) = x.divmod(y)
        let zero = BigInt.from_int(0)
        if q * y + r != x or r < zero or not (r < y.abs()): bad = "op {k}: BigInt {x} divmod {y}"
        elif (x * y) / y != x or x + y - y != x or (x - y) + y != x: bad = "op {k}: BigInt {x} * / {y}"
        elif BigInt.from_str(x.to_str()) != x: bad = "op {k}: BigInt round trip {x}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
