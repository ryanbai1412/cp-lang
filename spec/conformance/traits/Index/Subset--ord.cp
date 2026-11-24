// expect: ok, output "6 8"
domain B = 0..3
print(Subset<B>.single(B(1)).with(B(2)).ord, Subset<B>.count)
