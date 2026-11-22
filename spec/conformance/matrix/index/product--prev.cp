// expect: ok, output "YES"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
print(yes(D((0, 1)).prev! == D((0, 0)) and D((0, 0)).prev == None))
