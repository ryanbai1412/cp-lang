// expect: ok, output "3"
domain V = 0..5
fn f():
    let y = V.at(3) else: return
    print(y)
f()
