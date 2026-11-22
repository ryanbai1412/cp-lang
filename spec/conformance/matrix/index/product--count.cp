// expect: ok, output "1"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(D.count(x => x == D((1, 2))))
