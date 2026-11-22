// expect: ok, output "4"
let xs = [4, 2]
fn f():
    var left = 1
    while let y = (if left > 0: xs.first else: None):
        left -= 1
        print(y)
f()
