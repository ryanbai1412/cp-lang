// expect: ok, output "1 9 0 5\n10 2 1\n3 4 5\nba 2"
// Record update copies a record with some fields replaced (R02.5.3): `e with f: v` and
// `(..e, f: v)`, the base read once, the new values in the order written.
let a = (lo: 1, hi: 5)
let b = a with hi: 9
let c = (..a, lo: 0)
print(b.lo, b.hi, c.lo, c.hi)
struct P(x: Int, y: Int)
var o: ?P = P(1, 2)
let q = o! with x: 10
print(q.x, q.y, o!.x)
var log = ""
fn note(s: Str, v: Int) -> Int:
    log = log + s
    v
let r = (xs: [4, 5], k: 2)
let s = r with k: 3
print(s.k, s.xs)
let t = (u: 0, v: 0) with v: note("b", 2), u: note("a", 1)
print(log, t.v)
