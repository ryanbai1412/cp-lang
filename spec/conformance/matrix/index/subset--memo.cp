// expect: ok, output "7"
domain B = 0..3
@memo(map) fn f(x: Subset<B>) -> Int = 7
print(f(Subset<B>.full))
