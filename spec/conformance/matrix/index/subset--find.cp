// expect: ok, output "YES"
domain B = 0..3
print(yes(Subset<B>.find(x => x == Subset<B>.full)! == Subset<B>.full))
