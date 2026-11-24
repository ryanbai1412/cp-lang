// expect: ok, output "ok"
// input: "7 2000\n"
// model: Concave
// The slope treap against the O(n) slope list it replaced: the same falling slopes in a plain
// sorted list, where a value walks the list and `prefix_max`/`suffix_max` rewrite it.
use std.rand.{Rng}
use std.slope.{Concave}

struct Model:
    var lo: Int
    var base: Int
    var slopes: List<Int>

    fn at(self, c: Int) -> Int:
        var v = base
        for i in 0..c - lo: v += slopes[i]
        v

    fn max(self) -> Int:
        var best = base
        for s in slopes:
            if s <= 0: break
            best += s
        best

    fn count_at_least(self, v: Int) -> Int:
        var k = 0
        while k < slopes.len and slopes[k] >= v: k += 1
        k

    fn argmax(self) -> Range<Int> = lo + self.count_at_least(1)..lo + self.count_at_least(0) + 1

    fn insert(mut self, x: Int):
        var i = 0
        while i < slopes.len and slopes[i] >= x: i += 1
        slopes.insert(i, x)

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var f = Concave.flat(0..=40)
var m = Model(0, 0, List.filled(40, 0))
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 7)
    let a = rng.int(m.lo - 3, m.lo + m.slopes.len + 3)
    if op == 0:
        f.add_left(a)
        let j = if a <= m.lo: 0 else: min(a - m.lo, m.slopes.len)
        m.base += min(0, m.lo - a)
        for i in 0..j: m.slopes[i] += 1
    elif op == 1:
        f.add_right(a)
        let j = if a <= m.lo: 0 else: min(a - m.lo, m.slopes.len)
        m.base += min(0, a - m.lo)
        for i in j..m.slopes.len: m.slopes[i] -= 1
    elif op == 2:
        let s = rng.int(-3, 3)
        f.add_linear(s)
        m.base += s * m.lo
        for i in 0..m.slopes.len: m.slopes[i] += s
    elif op == 3:
        f.prefix_max()
        for i in m.count_at_least(0)..m.slopes.len: m.slopes[i] = 0
    elif op == 4:
        f.suffix_max()
        let j = m.count_at_least(1)
        m.base = m.max()
        for i in 0..j: m.slopes[i] = 0
    elif op == 5 and m.slopes.len < 400:
        f.slide_max(-1, 2)
        m.lo -= 1
        for _ in 0..3: m.insert(0)
    elif op == 6:
        let d = rng.int(-3, 3)
        f.shift(d)
        m.lo += d
    elif op == 7:
        let p = rng.int(0, 2)
        f.add_quadratic(x => -p * x * x + x)
        let q0 = -p * m.lo * m.lo + m.lo
        m.base += q0
        for i in 0..m.slopes.len:
            let x = m.lo + i
            m.slopes[i] += -p * (x + 1) * (x + 1) + (x + 1) - (-p * x * x + x)
    let hi = m.lo + m.slopes.len
    if f.lo != m.lo or f.hi != hi: bad = "op {k}: domain {f.lo}..={f.hi} want {m.lo}..={hi}"
    let c = rng.int(m.lo, hi)
    if bad == "" and f.at(c) != m.at(c): bad = "op {k}: at({c}) = {f.at(c)} want {m.at(c)}"
    if bad == "" and f.at(hi) != m.at(hi): bad = "op {k}: at({hi}) = {f.at(hi)} want {m.at(hi)}"
    if bad == "" and f.max() != m.max(): bad = "op {k}: max {f.max()} want {m.max()}"
    if bad == "" and f.argmax() != m.argmax(): bad = "op {k}: argmax {f.argmax()} want {m.argmax()}"
print(if bad == "": "ok" else: bad)
