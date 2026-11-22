// expect: ok, output "4"
let xs = [4, 2]
fn f():
    if let y = xs.first:
        print(y)
f()
