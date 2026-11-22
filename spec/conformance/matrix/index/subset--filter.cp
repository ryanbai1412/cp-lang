// expect: ok, output "7"
domain B = 0..3
print(Subset<B>.filter(x => x != Subset<B>.full).count())
