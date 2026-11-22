// expect: ok, output "3"
domain V = 0..5
fn f():
    if let y = V.at(3):
        print(y)
f()
