// expect: ok, output "3"
enum D = N | E | S | W
print(D.filter(x => x != D.E).count())
