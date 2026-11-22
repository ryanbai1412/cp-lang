# Cheerio 1 S3: count pairs i < j where, looking right from point i, point j sets a new steepest or
# shallowest slope (ties count). Slopes are exact ratios.
let pts = read [n] (x: Int, y: Int)
print(pts.keys.sum(of: i =>
    let slope = pts[i.r..].map(p => Ratio(p.y - pts[i].y, p.x - pts[i].x))
    zip(slope, slope.scan(max), slope.scan(min)).count((s, hi, lo) => s == hi or s == lo)))
