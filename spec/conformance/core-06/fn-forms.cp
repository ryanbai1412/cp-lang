// expect: ok, output "10 0"
// rule: 06.1.1
fn sq(x: Int) -> Int = x * x
fn f(x: Int) -> Int:
    if x < 0: return 0
    let y = sq(x)
    y + 1
print(f(3), f(-1))
