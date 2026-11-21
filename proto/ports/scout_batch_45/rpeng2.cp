// RPENG 2 (IOI 1998 Polygon): cut one edge, then merge neighbours by their edge's + or x
// original: corpus/submissions/4810287.cpp
let n: 1..=50 = read()
domain I = 0..n
let ring: [I](op: Char, val: Int) = read()      // edge i joins vertex i - 1 and vertex i

// span(i, len): (least, most) result of the chain of len + 1 vertices that
// starts at vertex i and walks clockwise around the ring
@memo
fn span(i: I, len: 0..n) -> (lo: Int, hi: Int):
    if len == 0: return (ring[i].val, ring[i].val)
    var (lo, hi) = (inf, -inf)
    for left in 0..len:
        let j = I.wrap(i + left + 1)
        let (a, b) = (span(i, left), span(j, len - left - 1))
        let ends = if ring[j].op == 'x': [a.lo * b.lo, a.lo * b.hi, a.hi * b.lo, a.hi * b.hi]
                   else: [a.lo + b.lo, a.hi + b.hi]
        lo min= ends.min()
        hi max= ends.max()
    (lo, hi)

let best = max(span(i, n - 1).hi for i in I)
print(best)
print([i + 1 for i in I if span(i, n - 1).hi == best])
