// expect: ok, output "YES"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(yes(D.min(by: x => 0)! == D((0, 0))))
