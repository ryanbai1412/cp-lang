// expect: ok, output "1"
domain B = 0..3
print(Subset<B>.count(x => x == Subset<B>.full))
