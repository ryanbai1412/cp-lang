// expect: ok, output "ok"
// rule: std.lazy.SeqFold.reverse
// `SeqFold` over a non-commutative fold (affine maps under composition) with a negating tag agrees
// with a plain list under push, insert_at, apply, reverse, fold, get and total, the operations an
// implicit treap of matrices (noi21p5) runs.
use std.lazy.{SeqFold}
use std.algebra.{Action, Monoid}

type Z = Mod<1_000_003>
type Aff = (Z, Z)

fn then(f: Aff, g: Aff) -> Aff = (f.0 * g.0, f.1 * g.0 + g.1)
fn neg(f: Aff) -> Aff = (f.0, Z(0) - f.1)

let comp = Monoid<Aff>((Z(1), Z(0)), then)
let negate = Action<Aff, Bool>(id: false, compose: (f, g) => f != g, apply: (f, x, _) => if f: neg(x) else: x, over: comp)

fn brute(xs: List<Aff>, lo: Int, hi: Int) -> Aff:
    var acc = (Z(1), Z(0))
    for i in lo..hi: acc = then(acc, xs[i])
    acc

var ok = true
for round in 0..40:
    var xs: List<Aff> = []
    var s = SeqFold<Aff, Bool>(comp, act: Some(negate))
    for _ in 0..300:
        let k = rand(0, 5)
        if k == 0 or xs.len == 0:
            let x = (Z(rand(1, 1000)), Z(rand(0, 1000)))
            s.push(x)
            xs.push(x)
        elif k == 1:
            let p = rand(0, xs.len)
            let x = (Z(rand(1, 1000)), Z(rand(0, 1000)))
            s.insert_at(p, x)
            xs.insert(p, x)
        else:
            let lo = rand(0, xs.len)
            let hi = rand(lo, xs.len)
            if k == 2:
                s.reverse(lo..hi)
                var mid = [xs[i] for i in lo..hi]
                mid.reverse()
                for i in lo..hi: xs[i] = mid[i - lo]
            elif k == 3:
                s.apply(lo..hi, true)
                for i in lo..hi: xs[i] = neg(xs[i])
            elif s.fold(lo..hi) != brute(xs, lo, hi): ok = false
        if s.total != brute(xs, 0, xs.len) or s.len != xs.len: ok = false
    for i in 0..xs.len:
        if s.get(i) != xs[i]: ok = false
print(if ok: "ok" else: "mismatch")
