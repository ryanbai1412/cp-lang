// A param a struct literal stores at its last read is taken by value and
// moved into the field, and a caller's dead local moves into it; one read
// after the literal is still copied.
struct R:
    var n: Int
    vals: List<Int>
    keys: Range<Int>
    pub fn new(a: List<Int>) -> R:
        let n = a.len
        let keys = a.keys
        var q = R(n, a, keys)
        q.n += 1
        q
struct W(xs: List<Int>)
fn kept(a: List<Int>) -> Int:
    let w = W(a)
    w.xs.len + a.len
struct S(sa: List<Int>, r: R)
fn build(m: Int) -> S:
    let sa = [i for i in 0..m]
    let lcp = [i * 2 for i in 0..m]
    S(sa, R(lcp))
let s = build(4)
print(s.sa, s.r.vals, s.r.n, s.r.keys, kept(s.sa), s.sa.len)
