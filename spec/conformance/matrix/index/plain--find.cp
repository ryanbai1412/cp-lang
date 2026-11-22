// expect: ok, output "YES"
domain D = 0..4
print(yes(D.find(x => x == D(1))! == D(1)))
