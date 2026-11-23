// Source: The 5th Universal Cup, Stage 5 (Grand Prix of Chengdu), problem B (Bouncing
//   Particles), qoj.ac/problem/23593
// Problem: multitest. In an n x n grid (n <= 1e9) four particles start at (x0, y0) moving
//   diagonally; each step every particle flips any direction that would leave the grid, then
//   splits into a horizontal and a vertical step child. q queries (sum q <= 1e5): count cells
//   on the line Ax + By + C = 0 (|A|, |B|, |C| <= 1e9) holding a particle at time t <= 1e9.
// Solution: the occupied cells at time t are the diamond |dx| + |dy| = t around the start,
//   folded into [0, n-1] with period 2(n-1) (mirror reflections). Swap x and y so B != 0;
//   walk each diamond side for min(t, 2(n-1)) steps, cut where it crosses a mirror line, so
//   each piece folds to a straight diagonal segment: it meets the line in one lattice point or,
//   if parallel and on it, in a whole x-interval. The answer is the length of the union of
//   these x-intervals (sorted sweep); O(1) pieces, so O(1) per query up to the sort.

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
    var reach: ?Int = None
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
            cur += dir * len
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
