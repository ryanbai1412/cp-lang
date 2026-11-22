// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
print(xs ?? List<Int>())
