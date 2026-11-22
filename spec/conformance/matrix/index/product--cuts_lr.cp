// expect: ok, output "1 2"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(Int(D((0, 1)).l), Int(D((0, 1)).r))
