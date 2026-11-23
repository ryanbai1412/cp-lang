// CF 1263E Editor: a segment tree of bracket balance, lowest and highest prefix.
struct Bal(sum: Int, lo: Int, hi: Int)
let n: 1..=1e6 = read()
let s: Str = read()
let flat = Bal(0, 0, 0)
fn join(a: Bal, b: Bal) -> Bal = Bal(a.sum + b.sum, min(a.lo, a.sum + b.lo), max(a.hi, a.sum + b.hi))
fn leaf(c: Char) -> Bal = if c == '(': Bal(1, 0, 1) elif c == ')': Bal(-1, -1, 0) else: flat
domain Pos = 0..=n
var t = SegTree<Pos, Bal>(Monoid(id: flat, op: join))
var cur = Pos.first!
var out: List<Int> = []
for c in s:
    if c == 'L': cur = cur.prev ?? cur
    elif c == 'R': cur = cur.next!
    else: t[cur] = leaf(c)
    let a = t.all
    out.push(if a.sum == 0 and a.lo >= 0: a.hi else: -1)
print(out)
