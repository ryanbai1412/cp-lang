// expect: ok, output "5"
domain B = 0..4
print(Int(Subset<B>.single(B(0)) | Subset<B>.single(B(2))))
