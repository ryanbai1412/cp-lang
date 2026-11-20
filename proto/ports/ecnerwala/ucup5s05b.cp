// Universal Cup 5 Stage 5 (Chengdu) B: a light pulse leaves (x0, y0) in an
// n x n mirrored box; after t steps its front is the diamond |dx| + |dy| = t
// folded back into the box.  Count the lattice points of the folded front on
// the line a*x + b*y + c = 0.  Each diamond side is cut where it crosses a
// mirror, so every piece folds to a straight diagonal segment; a piece meets
// the line in one point or, when parallel and on it, in a whole x-interval.
// Sweeping the x-events counts distinct points (b != 0 after the swap, so x
// determines the point).
fn sign(x: Int) -> Int = Int(x > 0) - Int(x < 0)

// Reflect a coordinate back into 0..=m.
fn fold(v: Int, m: Int) -> Int:
    let r = v % (2 * m)
    if r > m: 2 * m - r else: r

// Steps until a coordinate moving from s towards e hits a mirror line (a multiple of m).
fn to_mirror(s: Int, e: Int, m: Int) -> Int:
    let d = if e > s: -s else: s
    let gap = d % m
    min(if gap == 0: m else: gap, abs(s - e))

fn front(n: Int, px: Int, py: Int, qa: Int, qb: Int, qc: Int, t: Int) -> Int:
    if t == 0: return Int(qa * px + qb * py + qc == 0)
    // shift to 0-based coordinates, and make the y coefficient non-zero
    let m = n - 1
    let c = qc + qa + qb
    let (a, b, x0, y0) = if qb == 0: (qb, qa, py - 1, px - 1) else: (qa, qb, px - 1, py - 1)

    var evts: List<(Int, Int)> = []
    let corners = [(x0 + t, y0), (x0, y0 + t), (x0 - t, y0), (x0, y0 - t)]
    for z in 0..4:
        var (sx, sy) = corners[z]
        let (ex, ey) = corners[(z + 1) % 4]
        let (dx, dy) = (sign(ex - sx), sign(ey - sy))
        // the folded side is periodic with period 2m
        var left = min(t, 2 * m)
        while left > 0:
            let len = min(to_mirror(sx, sx + dx * left, m), to_mirror(sy, sy + dy * left, m))
            let (ux, uy) = (fold(sx, m), fold(sy, m))
            sx += dx * len
            sy += dy * len
            left -= len
            let (vx, vy) = (fold(sx, m), fold(sy, m))
            let at = ux * a + uy * b + c
            let slope = sign(vx - ux) * a + sign(vy - uy) * b
            if slope == 0:
                if at == 0:
                    evts.push((min(ux, vx), 1))
                    evts.push((max(ux, vx) + 1, -1))
            elif at % slope == 0 and 0 <= -at / slope and -at / slope <= len:
                let x = ux + sign(vx - ux) * (-at / slope)
                evts.push((x, 1))
                evts.push((x + 1, -1))

    var (open, prev, ans) = (0, 0, 0)
    for (x, d) in sorted(evts):
        if open > 0: ans += x - prev
        open += d
        prev = x
    ans

@!multitest
let (n, x0, y0, q): (2..=1e9, 1..=1e9, 1..=1e9, 1..=2e5) = read()
for _ in 0..q:
    let (a, b, c, t): (Int, Int, Int, Int) = read()
    print(front(n, x0, y0, a, b, c, t))
