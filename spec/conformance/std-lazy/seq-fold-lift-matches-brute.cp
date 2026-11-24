// expect: ok, output "ok"
// rule: std.lazy.SeqFold.new
// `lift:` keeps small elements (`-9..=9`) and folds each as `MaxSub.of`: under an assign tag
// the sequence agrees with a plain list under insert_many, erase_range, split_at, splice,
// apply, reverse and fold. A tag on one element lives beside it, so split and splice carry it
// over, and reads give `MaxSub` folds. Every other round mirrors one fold per node.
use std.lazy.{SeqFold}
use std.algebra.{Action, MaxSub}

type Val = -9..=9

let assign = Action<MaxSub, ?Val>.new(
    id: None,
    compose: (g, f) => g ?? f,
    apply: (f, x, len) => if let c = f: MaxSub.of(c).repeat(len) else: x,
)

fn brute(xs: List<Val>, lo: Int, hi: Int) -> MaxSub:
    var acc = MaxSub.zero
    for i in lo..hi: acc = acc + MaxSub.of(xs[i])
    acc

var ok = true
for round in 0..30:
    var xs: List<Val> = [rand(-9, 9) for _ in 0..rand(0, 6)]
    let mirror: ?(MaxSub -> MaxSub) = if round % 2 == 0: Some(x => x.reversed()) else: None
    var s = SeqFold<MaxSub, ?Val, Val>(+, xs, act: assign, mirror:, lift: MaxSub.of)
    for _ in 0..200:
        let k = rand(0, 6)
        let lo = rand(0, xs.len)
        let hi = rand(lo, xs.len)
        if k == 0:
            let ys: List<Val> = [rand(-9, 9) for _ in 0..rand(0, 5)]
            s.insert_many(lo, ys)
            xs = xs[0..lo] + ys + xs[lo..xs.len]
        elif k == 1:
            s.erase_range(lo..hi)
            xs = xs[0..lo] + xs[hi..xs.len]
        elif k == 2:
            var tail = s.split_at(lo)
            if s.len != lo or tail.len != xs.len - lo: ok = false
            s.splice(0, mut tail)
            xs = xs[lo..xs.len] + xs[0..lo]
        elif k == 3:
            let c: Val = rand(-9, 9)
            s.apply(lo..hi, Some(c))
            for i in lo..hi: xs[i] = c
        elif k == 4:
            s.reverse(lo..hi)
            var mid = xs[lo..hi]
            mid.reverse()
            for i in lo..hi: xs[i] = mid[i - lo]
        elif s.fold(lo..hi) != brute(xs, lo, hi): ok = false
        if s.total != brute(xs, 0, xs.len) or s.len != xs.len: ok = false
    if [x for x in s.items] != [MaxSub.of(x) for x in xs]: ok = false
print(if ok: "ok" else: "mismatch")
