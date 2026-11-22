// expect: ok, output "0"
// rule: 01.6.2
fn f(x: ?Int) -> Int:
    let v = x else: return 0
    v
print(f(None))
