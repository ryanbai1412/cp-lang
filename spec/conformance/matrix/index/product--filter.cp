// expect: ok, output "5"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(D.filter(x => x != D((1, 2))).count())
