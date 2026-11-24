// expect: ok, output "ok"
// input: "1 300\n"
// model: Monoid
// model: Comm
// model: Group
// model: Semigroup
// model: MaxSub
// model: MinCount
// model: MaxCount
// model: Affine
// model: Each
// model: each
// model: desc
// model: fold
// model: pow
// model: point_action
// model: cmp
// model: minmax
// model: opt
// std_functions: 2
use std.rand.{Rng}
use std.algebra.{Affine, Group, MaxCount, MaxSub, MinCount, Monoid, Semigroup, cmp, desc, each, fold, minmax, point_action, pow}

const P = 1000003

fn best_run(xs: List<Int>) -> ?Int:
    var best: ?Int = None
    for i in 0..xs.len:
        var s = 0
        for j in i..xs.len:
            s += xs[j]
            if best == None or s > best!: best = Some(s)
    best

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var bad = ""
for k in 0..n:
    if bad != "": break
    let xs = [rng.int(-9, 9) for _ in 0..rng.int(0, 6)]
    let op = rng.int(0, 5)
    if op == 0:
        var s = 0
        var hi = Int.MIN
        var lo = Int.MAX
        var pr = 1
        for x in xs:
            s += x
            hi = max(hi, x)
            lo = min(lo, x)
            pr *= x
        if fold(+, xs) != s or fold(max, xs) != hi or fold(min, xs) != lo or fold(*, xs) != pr: bad = "op {k}: fold over {xs}"
        let mine = Monoid<Int>(0, (a, b) => a + b)
        if bad == "" and (fold(mine, xs) != s or mine.monoid().op(3, 4) != 7 or mine.commutative().monoid().id != 0 or mine.commutative().op(2, 5) != 7):
            bad = "op {k}: a Monoid of your own over {xs}"
        let g = Group<Int>.of(+)
        let x = rng.int(-9, 9)
        if bad == "" and (g.inv(x) != -x or g.op(x, g.inv(x)) != g.id or Group<Int>.of(^).inv(x) != x): bad = "op {k}: Group inverse of {x}"
        let (e, hm) = fold(each(+, max), [(x, x) for x in xs])
        if bad == "" and (e != s or hm != hi): bad = "op {k}: each(+, max) over {xs} = ({e}, {hm})"
    elif op == 1:
        let m = fold(MaxSub.monoid(), [MaxSub.of(x) for x in xs])
        let want = best_run(xs)
        if m.try_best != want: bad = "op {k}: MaxSub over {xs} = {m.try_best ?? -99} want {want ?? -99}"
        let c = rng.int(0, 3)
        let sc = fold(MaxSub.monoid(), [MaxSub.of(x * c) for x in xs])
        if bad == "" and m.scale(c).try_best != sc.try_best: bad = "op {k}: MaxSub.scale({c}) over {xs}"
        if bad == "" and not xs.is_empty and (m.total != xs.sum() or (MaxSub.of(xs[0]) + MaxSub.of(-1)).try_best != Some(max(xs[0], -1))): bad = "op {k}: MaxSub total or + over {xs}"
        let mc = fold(MinCount.monoid(), [MinCount.of(x) for x in xs])
        let xc = fold(MaxCount.monoid(), [MaxCount.of(x) for x in xs])
        if bad == "" and not xs.is_empty:
            let lo = xs.min()!
            let hi = xs.max()!
            if mc.val != lo or mc.count != xs.filter(x => x == lo).count(): bad = "op {k}: MinCount over {xs} = ({mc.val}, {mc.count})"
            elif xc.val != hi or xc.count != xs.filter(x => x == hi).count(): bad = "op {k}: MaxCount over {xs} = ({xc.val}, {xc.count})"
        elif bad == "" and (mc.count != 0 or xc.count != 0 or MinCount<Int>.zero.count != 0): bad = "op {k}: an empty count is not 0"
    elif op == 2:
        let fs = [Affine(rng.int(-3, 3), rng.int(-5, 5)) for _ in 0..rng.int(0, 4)]
        let x = rng.int(-5, 5)
        var want = x
        for f in fs: want = f.a * want + f.b
        let got = fold(Affine.monoid(), fs).eval(x)
        if got != want: bad = "op {k}: Affine fold applied to {x} = {got} want {want}"
        let t = rng.int(0, 5)
        if bad == "" and fs.len > 0:
            var w = x
            for _ in 0..t: w = fs[0].a * w + fs[0].b
            let pw = pow(fs[0], t, op: Affine.monoid())
            if pw.eval(x) != w: bad = "op {k}: pow(Affine({fs[0].a}, {fs[0].b}), {t}) = {pw.eval(x)} want {w}"
        if bad == "" and Affine<Int>.id().eval(x) != x: bad = "op {k}: Affine.id"
        if bad == "" and fs.len >= 2 and Affine.then(fs[0], fs[1]).eval(x) != fs[1].eval(fs[0].eval(x)): bad = "op {k}: Affine.then"
    elif op == 3:
        let x = rng.int(-9, 9)
        let t = rng.int(0, 6)
        var mul = 1
        for _ in 0..t: mul = mul * x % P
        if pow(x, t, op: +) != x * t: bad = "op {k}: pow({x}, {t}, op: +) = {pow(x, t, op: +)}"
        elif pow(x, t, op: Monoid<Int>(1, (a, b) => a * b % P)) != mul: bad = "op {k}: pow({x}, {t}, op: *)"
        let ys = [if rng.chance(0.3): None else: Some(rng.int(0, 9)) for _ in 0..rng.int(0, 5)]
        var want: ?Int = None
        var left: ?Int = None
        for y in ys:
            if let v = y:
                want = if let w = want: Some((w * 10 + v) % P) else: Some(v)
                left = if let w = left: Some(w - v) else: Some(v)
        let f = (a: Int, b: Int) => (a * 10 + b) % P
        if bad == "" and fold(f.opt(), ys) != want: bad = "op {k}: opt fold = {fold(f.opt(), ys) ?? -1} want {want ?? -1}"
        elif bad == "" and fold(Semigroup((a: Int, b: Int) => a - b).opt(), ys) != left: bad = "op {k}: Semigroup.opt fold"
    elif op == 4:
        let a = rng.int(-3, 3)
        let b = rng.int(-3, 3)
        let want = if a < b: Ordering.Less elif a > b: Ordering.Greater else: Ordering.Equal
        if cmp(a, b) != want or a.cmp(b) != want: bad = "op {k}: cmp({a}, {b})"
        elif minmax(a, b) != (min(a, b), max(a, b)): bad = "op {k}: minmax({a}, {b})"
        elif (desc(a) < desc(b)) != (b < a): bad = "op {k}: desc({a}) < desc({b})"
        let sorted = xs.sorted(by: x => desc(x))
        if bad == "" and sorted != xs.sorted().rev.collect(): bad = "op {k}: sorted by desc {xs} = {sorted}"
    else:
        let act = point_action(id: 0, compose: (f: Int, g: Int) => f + g, apply: (f: Int, x: Int) => x + f, over: max)
        let f = rng.int(-5, 5)
        let g = rng.int(-5, 5)
        let x = rng.int(-9, 9)
        let span = rng.int(1, 6)
        if act.apply(f, x, span) != x + f or act.apply(act.compose(f, g), x, span) != x + f + g or act.over.op(f, g) != max(f, g):
            bad = "op {k}: point_action with tags {f}, {g} on {x}"
print(if bad == "": "ok" else: bad)
