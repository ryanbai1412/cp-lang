// expect: ok, output "11"
// rule: 05.10.7
fn f() -> Int:
    let g = (x: Int) -> Int =>
        if x > 0: return 1
        2
    g(5) + 10
print(f())
