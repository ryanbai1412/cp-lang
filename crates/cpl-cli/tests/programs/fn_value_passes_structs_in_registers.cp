// A fn value over 5-field scalar structs, as SegTree's Monoid holds lazy_assign's
// `join`: a plain fn is called straight through, a capturing one through its frame.
struct Run(a: 0..=1000, b: 0..=1000, c: 0..=1000, d: 0..=1000, e: 0..=1000)
fn join(x: Run, y: Run) -> Run = Run(x.a + y.a, x.b, max(x.c, y.c), y.d, x.e + 1)
fn fold(op: (Run, Run) -> Run, n: Int) -> Run:
    var r = Run(0, 1, 2, 3, 4)
    for i in 0..n: r = op(r, Run(1, i, i * 3, i, 0))
    r
let k = 7
fn shifted(x: Run, y: Run) -> Run = Run(x.a + k, y.b, x.c, x.d, y.e)
let r = fold(join, 5)
print(r.a, r.b, r.c, r.d, r.e)
let t = fold(shifted, 3)
print(t.a, t.b, t.c, t.d, t.e)
