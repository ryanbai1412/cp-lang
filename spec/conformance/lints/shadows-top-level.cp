// expect: L0203
let n = 5
fn f() -> Int:
    let n = 10 // <- here
    n
print(f() + n)
