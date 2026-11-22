// expect: ok, output "5"
let x: ?Int = 5
fn f():
    print(x ?? 0)
f()
