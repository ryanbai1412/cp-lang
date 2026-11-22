// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
fn f():
    match xs:
        Some(y) => print(y)
        None => print(0)
f()
