// expect: ok, output "7"
let (domain D, comp) = compress([30, 10, 20, 10])
@memo(map) fn f(x: D) -> Int = 7
print(f(D.last!))
