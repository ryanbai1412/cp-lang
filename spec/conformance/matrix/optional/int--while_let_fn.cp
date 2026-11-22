// expect: ok, output "5"
let x: ?Int = 5
fn f():
    var left = 1
    while let y = (if left > 0: x else: None):
        left -= 1
        print(y)
f()
