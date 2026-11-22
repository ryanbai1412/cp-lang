// expect: ok, output "3"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(D.count(x => x.step(Dir4.up()) != None))
