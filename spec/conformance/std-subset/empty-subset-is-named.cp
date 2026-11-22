// expect: ok, output "0 3"
domain B = 0..3
var best: [Subset<B>]Int = -1
best[Subset<B>.empty] = 0
best[Subset<B>.full] = 3
print(best[Subset<B>.empty], best[Subset<B>.full])
