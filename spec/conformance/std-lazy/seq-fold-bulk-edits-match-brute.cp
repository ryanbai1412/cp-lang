// expect: ok, output "ok"
// rule: std.lazy.SeqFold.erase_range
// `SeqFold` of max-subarray summaries under an assign tag agrees with a plain list under
// insert_many, erase_range, split_at, splice, apply, reverse and fold, the edits noi05p2 runs.
// Erased nodes are reused, so the run also covers recycled slots; every other round mirrors
// one fold per node (`mirror:`) instead of storing the backward fold.
use std.lazy.{SeqFold}
use std.algebra.{Action, MaxSub}

let assign = Action<MaxSub, ?Int>.new(
    id: None,
    compose: (g, f) => g ?? f,
    apply: (f, x, len) => if let c = f: pow(MaxSub.of(c), len, op: +) else: x,
)

fn brute(xs: List<Int>, lo: Int, hi: Int) -> MaxSub:
    var acc = MaxSub.zero
    for i in lo..hi: acc = acc + MaxSub.of(xs[i])
    acc

var ok = true
for round in 0..30:
    var xs: List<Int> = [rand(-9, 9) for _ in 0..rand(0, 6)]
    let mirror: ?(MaxSub -> MaxSub) = if round % 2 == 0: Some(x => x.reversed()) else: None
    var s = SeqFold<MaxSub, ?Int>(+, [MaxSub.of(x) for x in xs], act: assign, mirror:)
    for _ in 0..200:
        let k = rand(0, 6)
        let lo = rand(0, xs.len)
        let hi = rand(lo, xs.len)
        if k == 0:
            let ys = [rand(-9, 9) for _ in 0..rand(0, 5)]
            s.insert_many(lo, [MaxSub.of(y) for y in ys])
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
            let c = rand(-9, 9)
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
