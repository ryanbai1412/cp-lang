// expect: ok, output "4"
let xs = [4, 2]
fn f():
    print(xs.first ?? 0)
f()
