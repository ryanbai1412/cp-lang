// expect: ok, output "7"
domain D = 0..4
@memo(map) fn f(x: D) -> Int = 7
print(f(D(1)))
