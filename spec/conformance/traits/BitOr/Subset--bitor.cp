// expect: ok, output "2 YES"
domain B = 0..4
let a = Subset<B>.single(B(0)) | Subset<B>.single(B(2))
print(a.len, yes(B(2) in a))
