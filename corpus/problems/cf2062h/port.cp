// Merging stops exactly when the galaxies' bounding boxes have pairwise
// disjoint row and column projections, so a subset's value is its number of
// boxes. ways[hi][lo][cols] counts star subsets of rows lo..hi whose boxes all
// lie in the columns cols; galaxies[hi][lo][cols] sums their box counts.
type Z = Mod<1_000_000_007>
@!multitest
let n: 1..=14 = read()
domain Row = 0..n
domain Col = 0..n
let grid: [Row]Str = read()
var stars: [Row]Subset<Col> = Subset<Col>.empty
for r in Row:
    for c in Col:
        if grid[r][c.to_idx] == '1': stars[r] = stars[r].with(c)
var ways: [0..=n][0..=n][Subset<Col>]Z = 0
var galaxies: [0..=n][0..=n][Subset<Col>]Z = 0
for hi in 0..=n:
    ways[hi][hi].fill(1)
    for mid in (0..hi).rev:
        for cols in Subset<Col>:
            ways[hi][mid][cols] += ways[hi - 1][mid][cols]
            galaxies[hi][mid][cols] += galaxies[hi - 1][mid][cols]
        for y_lo in (0..n).rev:
            var band = Subset<Col>.empty
            for y in y_lo..n:
                band = band.with(Col(y))
                let inside = sum((stars[Row(r)] & band).len for r in mid..hi)
                let one_box = Z(2) ** inside - ways[hi][mid][band]
                for lo in 0..=mid:
                    for rest in (Subset<Col>.full - band).subsets:
                        let (w, g) = (ways[mid][lo][rest], galaxies[mid][lo][rest])
                        ways[hi][lo][rest | band] += w * one_box
                        galaxies[hi][lo][rest | band] += (w + g) * one_box
print(galaxies[n][0][Subset<Col>.full])
