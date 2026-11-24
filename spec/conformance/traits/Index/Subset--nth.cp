// expect: ok, output "0 2"
domain B = 0..3
print(Subset<B>.nth(5).items.map(b => Int(b)).collect())
