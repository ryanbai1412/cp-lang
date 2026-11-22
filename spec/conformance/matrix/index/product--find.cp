// expect: ok, output "YES"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(yes(D.find(x => x == D((1, 2)))! == D((1, 2))))
