// cheerio1s3: count pairs i < j where j's slope from i ties the steepest or the shallowest
// slope from i seen so far. Slopes compare exactly as ratios.
let n = read<1..=5_000>()
let pts = read<[n](x: Int, y: Int)>()
var count = 0
for i in 0..n:
    var lo: ?Ratio = none
    var hi: ?Ratio = none
    for j in i + 1..n:
        let s = Ratio(pts[j].y - pts[i].y, pts[j].x - pts[i].x)
        lo min= s
        hi max= s
        if s == lo! or s == hi!:
            count += 1
print(count)
