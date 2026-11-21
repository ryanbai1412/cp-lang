// Codeforces 2062 H: Galaxy Generator (dream). Merging stops exactly when the
// galaxies' bounding boxes have pairwise disjoint row and column projections,
// so a subset's value is its number of boxes. ways[rows, cols] counts star
// subsets of `rows` whose boxes all lie in the column set `cols`, together
// with the total number of boxes over those subsets.
type Z = Mod<1e9 + 7>
@!multitest
let n: 1..=14 = read()
domain Row = 0..n
domain Col = 0..n
let star: [Row, Col]Bool = read(as: "01")
let stars = star.rect_counts()                       // O(1) count of any rectangle
type Tally = (ways: Z, boxes: Z)                     // Tally * Z and + are fieldwise

var dp: [Row.spans][Bits<Col>]Tally = (ways: 0, boxes: 0)
for hi in Row.cuts:
    dp[hi..hi] = (ways: 1, boxes: 0)
    for mid in Row.cuts.before(hi).rev:
        dp[mid..hi] += dp[mid..hi.prev]              // the last row holds no box: elementwise
        for band in Col.spans(order: .inner_first):
            let box = (rows: mid..hi, cols: band)
            let one = Z(2) ** stars[box] - dp[mid..hi][band.bits].ways
            for lo in Row.cuts.through(mid), rest in (~band.bits).subsets():
                let (w, b) = dp[lo..mid][rest]
                dp[lo..hi][rest | band.bits] += (ways: w, boxes: w + b) * one
print(dp[Row.all][Bits<Col>.full].boxes)
