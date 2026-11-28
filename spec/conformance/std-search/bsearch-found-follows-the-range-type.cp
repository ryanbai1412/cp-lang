// expect: ok
// rule: search-dp
// output: "4 4 1 1\n1.5000000000 2.0000000000 1\n"
// One `bsearch` spelling: the range's type picks what it finds, a cut on a range of positions
// and `?F64` on a `RealRange`, what `lo..hi` over `F64` is (decision 456).
domain V = 0..10
let v = (V.first!..=V.last!).bsearch(x => x.ord >= 4).r!.ord
let w = (V.first!..=V.last!).bsearch(x => x.ord <= 3, last: true).r!.ord
let c = (0..100).bsearch(x => x * x >= 50)
let d = (0..10).bsearch(x => x <= 3, last: true)
print(v, w, if c == (8..8).l: 1 else: 0, if d == (4..4).l: 1 else: 0)
let hi = (0.0..4.0).bsearch(x => x <= 1.5, last: true, steps: 60)!
print(hi, (0.0..8.0).bsearch(x => x >= 2.0)!, if (0.0..1.0).bsearch(x => x > 2.0) == None: 1 else: 0)
