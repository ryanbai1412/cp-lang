// expect: ok, output "1 5"
// rule: 09.2.3
var calls = 0
fn idx() -> Int:
    calls += 1
    return 1
let a = [4, 5, 6]
let v = a[idx()]
print(calls, v)
