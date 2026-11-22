// expect: ok, output "YES"
enum D = N | E | S | W
print(yes(D.find(x => x == D.E)! == D.E))
