// expect: E0301 at 3
domain B = 0..3
var best: [Subset<B>]Int = -1
best[{}] = 0
