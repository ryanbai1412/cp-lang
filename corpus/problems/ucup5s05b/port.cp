// Universal Cup 5 Stage 5 (Chengdu) B: a light pulse leaves (x0, y0) in an
// n x n mirrored box; after t steps its front is the diamond |dx| + |dy| = t
// folded back into the box.  Count the lattice points of the folded front on
// the line a*x + b*y + c = 0.  Each diamond side is cut where it crosses a
// mirror, so every piece folds to a straight diagonal segment; a piece meets
// the line in one point or, when parallel and on it, in a whole x-interval.
// The answer is the length of the union of those x-intervals (b != 0 after the
// swap, so x determines the point).

// Reflect a coordinate back into 0..=m.
fn fold(v: Int, m: Int) -> Int:
    let r = v % (2 * m)
    if r > m: 2 * m - r else: r

fn fold_pt(p: Pt, m: Int) -> Pt = Pt(fold(p.x, m), fold(p.y, m))

// Steps until a coordinate moving from s towards e hits a mirror line (a multiple of m).
fn to_mirror(s: Int, e: Int, m: Int) -> Int:
    let d = if e > s: -s else: s
    let gap = d % m
    min(if gap == 0: m else: gap, abs(s - e))

fn union_len(spans: List<(Int, Int)>) -> Int:
    var total = 0
    var reach: ?Int = none
    for (lo, hi) in spans.sorted():
        let from = max(lo, reach ?? lo)
        if hi > from: total += hi - from
        reach = max(hi, reach ?? hi)
    total

fn signs(p: Pt) -> Pt = Pt(Int(p.x.sign), Int(p.y.sign))

@!multitest
let (n, px, py, q): (2..=1e9, 1..=1e9, 1..=1e9, 1..=2e5) = read()

fn front(qa: Int, qb: Int, qc: Int, t: Int) -> Int:
    if t == 0: return Int(qa * px + qb * py + qc == 0)
    // shift to 0-based coordinates, and make the y coefficient non-zero
    let m = n - 1
    let c = qc + qa + qb
    let (normal, origin) = if qb == 0:
        (Pt(qb, qa), Pt(py - 1, px - 1))
    else:
        (Pt(qa, qb), Pt(px - 1, py - 1))

    let corners = [origin + Pt(t, 0), origin + Pt(0, t), origin + Pt(-t, 0), origin + Pt(0, -t)]
    var spans: List<(Int, Int)> = []
    for z in 0..4:
        var cur = corners[z]
        let dir = signs(corners[(z + 1) % 4] - cur)
        // the folded side is periodic with period 2m
        var left = min(t, 2 * m)
        while left > 0:
            let len = min(
                to_mirror(cur.x, cur.x + dir.x * left, m),
                to_mirror(cur.y, cur.y + dir.y * left, m),
            )
            let u = fold_pt(cur, m)
            cur = cur + dir * len
            left -= len
            let v = fold_pt(cur, m)
            let at = u.dot(normal) + c
            let step = signs(v - u)
            let slope = step.dot(normal)
            if slope == 0:
                if at == 0: spans.push((min(u.x, v.x), max(u.x, v.x) + 1))
            elif at % slope == 0:
                let k = -at / slope
                if 0 <= k and k <= len:
                    let x = u.x + step.x * k
                    spans.push((x, x + 1))
    union_len(spans)

repeat q:
    let (a, b, c, t): (Int, Int, Int, 0..) = read()
    print(front(a, b, c, t))
