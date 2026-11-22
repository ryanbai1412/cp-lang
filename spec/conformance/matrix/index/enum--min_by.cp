// expect: ok, output "YES"
enum D = N | E | S | W
print(yes(D.min(by: x => 0)! == D.N))
