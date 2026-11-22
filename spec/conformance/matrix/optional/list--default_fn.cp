// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
fn f():
    print(xs ?? List<Int>())
f()
