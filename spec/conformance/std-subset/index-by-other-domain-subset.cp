// expect: E0301 at 4
domain B = 0..3
domain C = 0..2
var best: [Subset<B>]Int = 0
print(best[Subset<C>.full])
