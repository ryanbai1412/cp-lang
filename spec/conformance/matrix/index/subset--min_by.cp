// expect: ok, output "YES"
domain B = 0..3
print(yes(Subset<B>.min(by: x => 0)! == Subset<B>.empty))
