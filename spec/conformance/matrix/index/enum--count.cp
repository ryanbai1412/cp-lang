// expect: ok, output "1"
enum D = N | E | S | W
print(D.count(x => x == D.E))
