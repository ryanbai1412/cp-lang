// expect: ok
// rule: search-dp
// output: "3.0000000000\n4.0000000000\n"
// Real searches are the integer ones on a `RealRange`, what `lo..hi` over `F64` is, with a step count.
print((0.0..8.0).bsearch(x => x >= 3.0, steps: 60)!)
print((0.0..8.0).valley(by: x => (x - 4.0) * (x - 4.0), steps: 200))
