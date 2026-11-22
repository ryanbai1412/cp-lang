// expect: ok, output "3"
domain V = 0..5
fn f():
    print(V.at(3) ?? V(0))
f()
