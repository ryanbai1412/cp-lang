// expect: ok, output "YES"
enum D = N | E | S | W
print(yes(D.E.prev! == D.N and D.N.prev == None))
