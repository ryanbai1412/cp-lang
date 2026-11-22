// expect: ok, output "YES"
enum D = N | E | S | W
print(yes(D.N.next! == D.E))
