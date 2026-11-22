// expect: ok, output "1 2"
enum D = N | E | S | W
print(Int(D.E.l), Int(D.E.r))
