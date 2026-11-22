// expect: ok, output "YES"
domain D = 0..4
print(yes(D(1).prev! == D(0) and D(0).prev == None))
