// expect: ok, output "1"
domain B = 0..3
print(Int(Subset<B>.single(B(0))))
