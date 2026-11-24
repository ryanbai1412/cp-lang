// Cheerio 1 S3: count pairs i < j where, looking right from point i, point j sets a new steepest or
// shallowest slope (ties count). Slopes are exact ratios.
let n: 2..=5000 = read()
let pts: [0..n](t: 0..=1e9, p: 0..=1e9) = read()
var pairs = 0
for i in 0..n:
    var hi: ?Ratio = None
    var lo: ?Ratio = None
    for j in i + 1..n:
        let s = Ratio(pts[j].p - pts[i].p, pts[j].t - pts[i].t)
        let top = hi.is_none_or(h => s >= h)
        let low = lo.is_none_or(l => s <= l)
        if top: hi = Some(s)
        if low: lo = Some(s)
        if top or low: pairs += 1
print(pairs)
