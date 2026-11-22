// expect: ok, output "5"
let x: ?Int = 5
fn f():
    if let y = x:
        print(y)
f()
