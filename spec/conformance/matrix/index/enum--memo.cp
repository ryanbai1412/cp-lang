// expect: ok, output "7"
enum D = N | E | S | W
@memo(map) fn f(x: D) -> Int = 7
print(f(D.E))
