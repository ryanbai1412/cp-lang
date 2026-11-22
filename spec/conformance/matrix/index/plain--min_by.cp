// expect: ok, output "YES"
domain D = 0..4
print(yes(D.min(by: x => 0)! == D(0)))
