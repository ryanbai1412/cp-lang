// expect: ok, output "ok"
// input: "1 300\n"
// model: gcd
// model: lcm
// model: ext_gcd
// model: isqrt
// model: ceil_sqrt
// model: iroot
// model: choose
// model: choose2
// model: ceil_div
// model: divmod
// model: bit
// model: bit_len
// model: is_pow2
// model: sign
// model: digits
// model: to_str
// model: fixed
// model: splitmix64
// The plain-Int helpers of std.math against loops: seeded random arguments (mostly small,
// sometimes near the edges), each fn compared with its obvious brute force. Reads `seed n`;
// prints the first op that differs.
use std.math.{ceil_sqrt, iroot, choose, choose2, ext_gcd, fixed, splitmix64, bit_len}
use std.rand.{Rng}

fn brute_gcd(a: Int, b: Int) -> Int:
    var g = 0
    for d in 1..=max(abs(a), abs(b)):
        if a % d == 0 and b % d == 0: g = d
    g

fn brute_root(x: Int, k: Int) -> Int:
    var r = 0
    while (r + 1) ** k <= x: r += 1
    r

fn brute_fact(n: Int) -> Int:
    var f = 1
    for i in 1..=n: f *= i
    f

fn brute_str(x: Int, base: Int) -> Str:
    let ds = "0123456789abcdefghijklmnopqrstuvwxyz"
    if x == 0: return "0"
    var out = ""
    var y = abs(x)
    while y > 0:
        out = "{ds[y % base]}" + out
        y = y / base
    if x < 0: "-" + out else: out

// Long division, one digit past `k`, then round half away from zero on the magnitude.
fn brute_fixed(num: Int, den: Int, k: Int) -> Str:
    let (n, d) = (abs(num), abs(den))
    var whole = n / d
    var rem = n % d
    var frac: List<Int> = []
    for _ in 0..k + 1:
        rem *= 10
        frac.push(rem / d)
        rem = rem % d
    var up = frac[k] >= 5
    var i = k - 1
    while up and i >= 0:
        frac[i] += 1
        up = frac[i] == 10
        if up: frac[i] = 0
        i -= 1
    if up: whole += 1
    let zero = whole == 0 and all(frac[j] == 0 for j in 0..k)
    var out = if (num < 0) != (den < 0) and not zero: "-" else: ""
    out += "{whole}"
    if k > 0:
        out += "."
        for j in 0..k: out += "{frac[j]}"
    out

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 11)
    let a = rng.int(-60, 60)
    let b = rng.int(-60, 60)
    if op == 0:
        let g = gcd(a, b)
        let e = ext_gcd(a, b)
        if g != brute_gcd(a, b) or e.g != g or a * e.x + b * e.y != g:
            bad = "op {k}: gcd/ext_gcd({a}, {b})"
        elif lcm(a, b) != (if a == 0 or b == 0: 0 else: abs(a * b) / g):
            bad = "op {k}: lcm({a}, {b})"
    elif op == 1:
        let x = if rng.chance(0.2): Int.MAX - rng.int(0, 1000) else: rng.int(0, 5000)
        let r = isqrt(x)
        let ok = r >= 0 and r * r <= x and (r == 3_037_000_499 or (r + 1) * (r + 1) > x)
        if not ok or (x <= 5000 and r != brute_root(x, 2)):
            bad = "op {k}: isqrt({x})"
        elif ceil_sqrt(x) != (if r * r == x: r else: r + 1):
            bad = "op {k}: ceil_sqrt({x})"
    elif op == 2:
        let x = rng.int(0, 5000)
        let e = rng.int(1, 6)
        if iroot(x, e) != brute_root(x, e): bad = "op {k}: iroot({x}, {e})"
    elif op == 3:
        let m = rng.int(-3, 20)
        let j = rng.int(-3, 22)
        let want = if m < 0 or j < 0 or j > m: 0 else: brute_fact(m) / brute_fact(j) / brute_fact(m - j)
        if choose(m, j) != want: bad = "op {k}: choose({m}, {j})"
        elif choose2(m) != (if m < 0: 0 else: m * (m - 1) / 2): bad = "op {k}: choose2({m})"
    elif op == 4 and b != 0:
        var q = -100
        while (b > 0 and q * b < a) or (b < 0 and q * b > a): q += 1
        let (dq, dr) = a.divmod(b)
        if a.ceil_div(b) != q: bad = "op {k}: ceil_div({a}, {b})"
        elif dr < 0 or dr >= abs(b) or dq * b + dr != a: bad = "op {k}: divmod({a}, {b})"
    elif op == 5:
        let x = if rng.chance(0.5): rng.int(Int.MIN, Int.MAX) else: a
        let i = rng.int(0, 70)
        var y = x
        for _ in 0..min(i, 63): y = (y - (y % 2)) / 2
        let want = if i >= 63: x < 0 else: y % 2 == 1
        if x.bit(i) != want: bad = "op {k}: bit({x}, {i})"
    elif op == 6:
        let x = rng.int(0, 1 << rng.int(0, 61))
        var len = 0
        while len < 63 and (1 << len) <= x: len += 1
        var p = 1
        while p < x: p *= 2
        if bit_len(x) != len: bad = "op {k}: bit_len({x})"
        elif x.is_pow2 != (x > 0 and p == x): bad = "op {k}: is_pow2({x})"
    elif op == 7:
        let want = if a > 0: 1 elif a < 0: -1 else: 0
        if Int(a.sign) != want or Int(a.sign) * abs(a) != a: bad = "op {k}: sign({a})"
    elif op == 8:
        let x = rng.int(0, 100000)
        let base = rng.int(2, 36)
        let ds = x.digits(base: base)
        var back = 0
        for d in ds.reversed(): back = back * base + d
        let ok = back == x and all(0 <= d < base for d in ds) and (ds.len == 1 or ds[ds.len - 1] != 0)
        if not ok: bad = "op {k}: digits({x}, {base})"
    elif op == 9:
        let x = if rng.chance(0.3): rng.int(Int.MIN + 1, Int.MAX) else: rng.int(-100000, 100000)
        let base = rng.int(2, 36)
        if x.to_str(base: base) != brute_str(x, base): bad = "op {k}: to_str({x}, {base})"
    elif op == 10 and b != 0:
        let num = rng.int(-2000, 2000)
        let kk = rng.int(0, 4)
        if fixed(num, b, kk) != brute_fixed(num, b, kk): bad = "op {k}: fixed({num}, {b}, {kk}) = {fixed(num, b, kk)} want {brute_fixed(num, b, kk)}"
    elif op == 11:
        let x = rng.int(Int.MIN, Int.MAX)
        var r = Rng(x)
        if splitmix64(U64(x)) != r.next(): bad = "op {k}: splitmix64({x})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
