// expect: E0302
// `{}` where a Subset<D> is expected is an empty block, not a Subset (decision 220); the fix writes `Subset<B>.empty`.
domain B = 0..3
var s: Subset<B> = {}  // <- here
print(s.len)
