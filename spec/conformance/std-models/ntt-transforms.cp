// expect: ok, output "ok"
// input: "1 300\n"
// model: fwht
// model: sos
// model: subset_exp
// model: subset_log
// std.ntt's mask transforms against sums over every mask: fwht against the signed sum, sos
// against submask and supermask scans (each inverse against the original), and the set power
// series exp and log against the partition recurrence on the lowest bit. Reads `seed n`;
// prints the first op that differs.
use std.ntt.{fwht, sos, subset_exp, subset_log, SosOver}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>

fn ones(x: Int) -> Int:
    var (c, y) = (0, x)
    while y > 0:
        c += y & 1
        y >>= 1
    c

fn low_bit(m: Int) -> Int:
    var b = 1
    while m & b == 0: b *= 2
    b

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 3)
    let len = 1 << rng.int(0, 6)
    let a = [rng.int(-100, 100) for _ in 0..len]
    if op == 0:
        let want = [sum((if ones(m & s) % 2 == 0: a[s] else: -a[s]) for s in 0..len) for m in 0..len]
        var x = a.clone()
        fwht(mut x)
        var y = x.clone()
        fwht(mut y, inverse: true)
        var z = [Z(v) for v in a]
        fwht(mut z)
        var zi = z.clone()
        fwht(mut zi, inverse: true)
        if x != want or z != [Z(v) for v in want]: bad = "op {k}: fwht({a})"
        elif y != a or zi != [Z(v) for v in a]: bad = "op {k}: fwht({x}, inverse: true)"
    elif op == 1:
        let sub = [sum(a[s] for s in 0..len if s & m == s) for m in 0..len]
        let sup = [sum(a[s] for s in 0..len if s & m == m) for m in 0..len]
        var (x, y) = (a.clone(), a.clone())
        sos(mut x)
        sos(mut y, over: SosOver.Sup)
        var (xi, yi) = (x.clone(), y.clone())
        sos(mut xi, inverse: true)
        sos(mut yi, over: SosOver.Sup, inverse: true)
        if x != sub: bad = "op {k}: sos({a})"
        elif y != sup: bad = "op {k}: sos({a}, over: Sup)"
        elif xi != a or yi != a: bad = "op {k}: sos inverse of {a}"
    elif op == 2:
        var x = [Z(rng.int(0, 998_244_352)) for _ in 0..len]
        x[0] = Z(0)
        var want = List.filled(len, Z(0))
        want[0] = Z(1)
        for m in 1..len:
            let lb = low_bit(m)
            var s = m
            while s > 0:
                if s & lb != 0: want[m] += x[s] * want[m ^ s]
                s = (s - 1) & m
        if subset_exp(x) != want: bad = "op {k}: subset_exp({x})"
        elif subset_log(want) != x: bad = "op {k}: subset_log({want})"
    else:
        var x = [Z(rng.int(0, 998_244_352)) for _ in 0..len]
        x[0] = Z(1)
        var want = List.filled(len, Z(0))
        for m in 1..len:
            let lb = low_bit(m)
            var v = x[m]
            var s = (m - 1) & m
            while s > 0:
                if s & lb != 0: v -= want[s] * x[m ^ s]
                s = (s - 1) & m
            want[m] = v
        if subset_log(x) != want: bad = "op {k}: subset_log({x})"
        elif subset_exp(want) != x: bad = "op {k}: subset_exp({want})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
