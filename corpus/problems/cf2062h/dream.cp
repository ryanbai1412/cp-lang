// Merging stops exactly when the galaxies' bounding boxes have pairwise
// disjoint row and column projections, so a subset's value is its number of
// boxes. ways[rows, cols] counts star subsets of `rows` whose boxes all lie in
// the column set `cols`, together with the total number of boxes over them.
type Z = Mod<1e9 + 7>
@!multitest
let n: 1..=14 = read()
domain Row = 0..n
domain Col = 0..n
let grid: [Row]Str = read()
let stars = Prefix<Row, Col, Int>([Row, Col] (r, c) => if grid[r][c.ord] == '1': 1 else: 0)

// ways/boxes[hi][lo][cols]: rows lo..hi split into boxes covering exactly `cols`
var ways: [Row.cuts][Row.cuts][Subset<Col>]Z = 0
var boxes: [Row.cuts][Row.cuts][Subset<Col>]Z = 0
for hi in Row.cuts:
    ways[hi][hi].fill(1)
    for mid in (Row.cuts.first!..hi).rev:
        for cols in Subset<Col>:                     // the last row holds no box
            ways[hi][mid][cols] += ways[hi.prev!][mid][cols]
            boxes[hi][mid][cols] += boxes[hi.prev!][mid][cols]
        for left in Col.rev:                         // narrower bands first
            var band = Subset<Col>.empty
            for right in left..=Col.last!:
                band = band.with(right)
                let one = Z(2) ** stars.fold(rows: mid..hi, cols: left..=right) - ways[hi][mid][band]
                for lo in Row.cuts.first!..=mid, rest in (Subset<Col>.full - band).subsets:
                    let (w, b) = (ways[mid][lo][rest], boxes[mid][lo][rest])
                    ways[hi][lo][rest | band] += w * one
                    boxes[hi][lo][rest | band] += (w + b) * one
print(boxes[Row.cuts.last!][Row.cuts.first!][Subset<Col>.full])
