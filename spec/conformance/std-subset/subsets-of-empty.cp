// expect: ok, output "1 1"
domain B = 0..3
let e: Subset<B> = Subset<B>.single(B(0)).without(B(0))
print(e.subsets.collect().len, Subset<B>.single(B(0)).with(B(1)).with(B(2)).supersets.collect().len)
