// expect: ok, output "2"
// rule: 05.10.1 06.2.1 06.2.4
fn f() -> Int:
    var c = 0
    let inc = () => c += 1
    inc()
    inc()
    c
print(f())
