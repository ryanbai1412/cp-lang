// expect: ok, output "1"
// rule: 02.8.5
fn f() -> Int:
    let a: [4000000]Int = [4000000].filled(1)
    a[3999999]
print(f())
