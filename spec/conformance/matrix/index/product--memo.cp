// expect: ok, output "7"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
@memo(map) fn f(x: D) -> Int = 7
print(f(D((1, 2))))
