// cheerio1s3: count pairs p < q where q's slope from p ties the steepest or the shallowest
// slope from p seen so far. Slopes compare exactly as ratios, and "so far" is a running min
// and max over the points after p.
let n = read<1..=5_000>()
domain P = 1..=n
let pts = read<[P](x: Int, y: Int)>()
fn ties(p: P) -> Int:
    let s = pts[p.r..].map(q => Ratio(q.y - pts[p].y, q.x - pts[p].x))
    let (lo, hi) = (s.scan(min), s.scan(max))
    s.keys.count(q => s[q] == lo[q] or s[q] == hi[q])
print(P.map(ties).sum())
