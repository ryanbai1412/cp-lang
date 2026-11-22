// expect: ok, output "YES"
domain B = 0..3
print(yes(Subset<B>.first! == Subset<B>.empty))
