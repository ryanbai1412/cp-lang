// NOIP 2002 P4, cover n points with at most k <= 3 disjoint axis-parallel rectangles
// original: corpus/submissions/5096459.cpp
let (n, k): (1..=50, 1..=3) = read()
let pts: List<Pt> = read(n)

fn area(ps: List<Pt>) -> Int:
    if ps.is_empty: return 0
    (max(p.x for p in ps)! - min(p.x for p in ps)!) * (max(p.y for p in ps)! - min(p.y for p in ps)!)

// least total area of at most `k` rectangles: cut with a vertical or a
// horizontal line between two distinct coordinates and share k across the sides
fn cover(ps: List<Pt>, k: Int) -> Int:
    if k == 1 or ps.is_empty: return area(ps)
    var best = area(ps)
    for axis in [p => (p.x, p.y), p => (p.y, p.x)]:
        let s = ps.sorted(by: axis)
        for i in 1..s.len if axis(s[i - 1]).0 < axis(s[i]).0:
            for j in 1..k: best min= cover(s[..i], j) + cover(s[i..], k - j)
    best

print(cover(pts, k))
