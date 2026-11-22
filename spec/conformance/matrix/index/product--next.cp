// expect: ok, output "YES"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(yes(D((0, 0)).next! == D((0, 1))))
