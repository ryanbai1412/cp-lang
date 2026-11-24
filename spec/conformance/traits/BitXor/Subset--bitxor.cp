// expect: ok, output "3 NO"
domain B = 0..4
let a = Subset<B>.full ^ Subset<B>.single(B(1))
print(a.len, yes(B(1) in a))
