// expect: ok, output "ok"
// input: "1 300\n"
// model: SeqFold
// SeqFold over affine maps under composition (a fold that does not commute) with a negating tag,
// against a List: seeded random push, insert_at, apply, reverse, fold and get, with the total and
// length compared after each op. Reads `seed n`; prints the first op that differs.
use std.lazy.{SeqFold}
use std.algebra.{Action, Monoid}

type Z = Mod<1_000_003>
type Aff = (Z, Z)

fn then(f: Aff, g: Aff) -> Aff = (f.0 * g.0, f.1 * g.0 + g.1)
fn neg(f: Aff) -> Aff = (f.0, Z(0) - f.1)

fn brute(a: List<Aff>, lo: Int, hi: Int) -> Aff:
    var acc = (Z(1), Z(0))
    for i in lo..hi: acc = then(acc, a[i])
    acc

let comp = Monoid<Aff>((Z(1), Z(0)), then)
let negate = Action<Aff, Bool>(id: false, compose: (f, g) => f != g, apply: (f, x, _) => if f: neg(x) else: x, over: comp)
let seed0 = read<Int>()
let n = read<Int>()
var seed = seed0
var s = SeqFold<Aff, Bool>(comp, act: Some(negate))
var a: List<Aff> = []
var bad = ""
for k in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let op = seed % 6
    let x = (Z(seed / 6 % 997 + 1), Z(seed / 6000 % 1000))
    seed = (seed * 1103515245 + 12345) % 2147483648
    let lo = if a.len == 0: 0 else: seed % (a.len + 1)
    let hi = lo + (seed / 4096) % (a.len - lo + 1)
    if op == 0 or a.len == 0:
        s.push(x)
        a.push(x)
    elif op == 1:
        s.insert_at(lo, x)
        a.insert(lo, x)
    elif op == 2:
        s.reverse(lo..hi)
        let mid = [a[hi - 1 - (i - lo)] for i in lo..hi]
        for i in lo..hi: a[i] = mid[i - lo]
    elif op == 3:
        s.apply(lo..hi, true)
        for i in lo..hi: a[i] = neg(a[i])
    elif op == 4:
        if s.fold(lo..hi) != brute(a, lo, hi): bad = "op {k}: fold({lo}..{hi})"
    elif lo < a.len:
        if s.get(lo) != a[lo]: bad = "op {k}: get({lo})"
    if bad == "" and (s.len != a.len or s.total != brute(a, 0, a.len)):
        bad = "op {k}: total or len"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
