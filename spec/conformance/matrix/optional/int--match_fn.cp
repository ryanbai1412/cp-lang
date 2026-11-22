// expect: ok, output "5"
let x: ?Int = 5
fn f():
    match x:
        Some(y) => print(y)
        None => print(0)
f()
