// expect: ok, output "5"
let x: ?Int = 5
fn f():
    let y = x else: return
    print(y)
f()
