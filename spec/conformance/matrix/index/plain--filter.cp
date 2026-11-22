// expect: ok, output "3"
domain D = 0..4
print(D.filter(x => x != D(1)).count())
