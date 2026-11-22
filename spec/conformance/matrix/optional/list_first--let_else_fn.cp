// expect: ok, output "4"
let xs = [4, 2]
fn f():
    let y = xs.first else: return
    print(y)
f()
