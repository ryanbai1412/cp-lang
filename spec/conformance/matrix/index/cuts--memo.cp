// expect: ok, output "7"
domain V = 0..3
@memo(map) fn f(x: V.cuts) -> Int = 7
print(f(V.cuts(1)))
