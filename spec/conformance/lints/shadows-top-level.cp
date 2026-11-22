// expect: L0203 at 3
let n = 5
fn f() -> Int:
    let n = 10
    n
print(f() + n)
