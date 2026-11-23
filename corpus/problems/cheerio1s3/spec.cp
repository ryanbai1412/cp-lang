// cheerio1s3: count pairs p < q where q's slope from p ties the steepest or the shallowest
// slope from p seen so far. Slopes compare exactly as ratios (t strictly increases, so the run
// is never 0), and "so far" is a running min and max over the points after p, starting from none.
let n = read<2..=5_000>()
domain P = 1..=n
let pts = read<[P](x: 0..=1e9, y: 0..=1e9)>()
fn ties(p: P) -> Int:
    var (lo, hi): (?Ratio, ?Ratio) = (None, None)
    var k = 0
    for q in p.after:
        let s = Ratio(pts[q].y - pts[p].y, pts[q].x - pts[p].x)
        if s <= (lo ?? s) or s >= (hi ?? s): k += 1
        lo min= s
        hi max= s
    k
print(P.map(ties).sum())
